use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::{GamestateKind, SaveMetadata, index};
use crate::paths::{STELLARIS_STEAM_APP_ID, default_save_games_dir};

const USERDATA_DIR_NAME: &str = "userdata";
const REMOTE_DIR_NAME: &str = "remote";

#[derive(Debug, Clone)]
pub struct ScannedSave {
    pub path: PathBuf,
    pub file_name: String,
    pub campaign_key: String,
    pub steam_user_id: Option<String>,
    pub metadata: Option<SaveMetadata>,
    pub modified_at_millis: u64,
    pub file_size: u64,
    pub gamestate_kind: Option<GamestateKind>,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SaveScanDiagnostic {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct SaveScanResult {
    pub saves: Vec<ScannedSave>,
    pub diagnostics: Vec<SaveScanDiagnostic>,
}

pub fn scan_local(documents_dir: &Path) -> SaveScanResult {
    scan_save_root(&default_save_games_dir(documents_dir), None)
}

pub fn scan_cloud(steam_libraries: &[PathBuf]) -> SaveScanResult {
    let mut result = SaveScanResult::default();
    for steam_root in steam_libraries {
        let userdata_dir = steam_root.join(USERDATA_DIR_NAME);
        if !userdata_dir.exists() {
            continue;
        }
        let user_dirs = match fs::read_dir(&userdata_dir) {
            Ok(entries) => entries,
            Err(error) => {
                result.diagnostics.push(SaveScanDiagnostic {
                    path: userdata_dir,
                    message: format!("无法读取 Steam 用户目录：{error}"),
                });
                continue;
            }
        };
        for user_dir in user_dirs {
            let user_dir = match user_dir {
                Ok(entry) => entry,
                Err(error) => {
                    result.diagnostics.push(SaveScanDiagnostic {
                        path: userdata_dir.clone(),
                        message: format!("无法读取 Steam 用户目录项：{error}"),
                    });
                    continue;
                }
            };
            let Ok(file_type) = user_dir.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }
            let steam_user_id = user_dir.file_name().to_string_lossy().into_owned();
            if steam_user_id.is_empty() || !steam_user_id.bytes().all(|byte| byte.is_ascii_digit())
            {
                continue;
            }
            let save_root = user_dir
                .path()
                .join(STELLARIS_STEAM_APP_ID)
                .join(REMOTE_DIR_NAME)
                .join(crate::paths::SAVE_GAMES_DIR_NAME);
            let mut user_result = scan_save_root(&save_root, Some(steam_user_id));
            result.saves.append(&mut user_result.saves);
            result.diagnostics.append(&mut user_result.diagnostics);
        }
    }
    result
}

fn scan_save_root(save_root: &Path, steam_user_id: Option<String>) -> SaveScanResult {
    let mut result = SaveScanResult::default();
    let campaign_dirs = match fs::read_dir(save_root) {
        Ok(entries) => entries,
        Err(error) => {
            result.diagnostics.push(SaveScanDiagnostic {
                path: save_root.to_path_buf(),
                message: format!("无法读取存档根目录：{error}"),
            });
            return result;
        }
    };

    for campaign_dir in campaign_dirs {
        let campaign_dir = match campaign_dir {
            Ok(entry) => entry,
            Err(error) => {
                result.diagnostics.push(SaveScanDiagnostic {
                    path: save_root.to_path_buf(),
                    message: format!("无法读取战役目录项：{error}"),
                });
                continue;
            }
        };
        let Ok(file_type) = campaign_dir.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let campaign_key = campaign_dir.file_name().to_string_lossy().into_owned();
        scan_campaign(
            &campaign_dir.path(),
            &campaign_key,
            steam_user_id.as_deref(),
            &mut result,
        );
    }
    result
}

fn scan_campaign(
    campaign_dir: &Path,
    campaign_key: &str,
    steam_user_id: Option<&str>,
    result: &mut SaveScanResult,
) {
    let save_files = match fs::read_dir(campaign_dir) {
        Ok(entries) => entries,
        Err(error) => {
            result.diagnostics.push(SaveScanDiagnostic {
                path: campaign_dir.to_path_buf(),
                message: format!("无法读取战役目录：{error}"),
            });
            return;
        }
    };

    for save_file in save_files {
        let save_file = match save_file {
            Ok(entry) => entry,
            Err(error) => {
                result.diagnostics.push(SaveScanDiagnostic {
                    path: campaign_dir.to_path_buf(),
                    message: format!("无法读取存档目录项：{error}"),
                });
                continue;
            }
        };
        let path = save_file.path();
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("sav"))
        {
            continue;
        }

        let file_name = save_file.file_name().to_string_lossy().into_owned();
        let file_metadata = match save_file.metadata() {
            Ok(metadata) if metadata.is_file() => metadata,
            Ok(_) => continue,
            Err(error) => {
                result.diagnostics.push(SaveScanDiagnostic {
                    path,
                    message: format!("无法读取存档文件属性：{error}"),
                });
                continue;
            }
        };
        let modified_at_millis = file_metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
            .unwrap_or_default();

        let (metadata, gamestate_kind, unavailable_reason) = match index(&path) {
            Ok(index) => {
                let reason = matches!(index.gamestate_kind, GamestateKind::Binary)
                    .then(|| "二进制或铁人存档暂不支持".to_string());
                (Some(index.metadata), Some(index.gamestate_kind), reason)
            }
            Err(error) => (None, None, Some(error.to_string())),
        };
        result.saves.push(ScannedSave {
            path,
            file_name,
            campaign_key: campaign_key.to_string(),
            steam_user_id: steam_user_id.map(str::to_string),
            metadata,
            modified_at_millis,
            file_size: file_metadata.len(),
            gamestate_kind,
            unavailable_reason,
        });
    }
}
