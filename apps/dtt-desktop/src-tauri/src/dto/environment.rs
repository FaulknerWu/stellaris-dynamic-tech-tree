use dtt_application::{DetectedEnvironment, ResolveEnvironmentRequest, ResolvedEnvironment};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use super::generation::SupportedLanguageDto;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ResolveEnvironmentRequestDto {
    #[ts(optional)]
    pub game_root: Option<String>,
    #[ts(optional)]
    pub documents_dir: Option<String>,
    #[ts(optional)]
    pub launcher_db: Option<String>,
}

impl From<ResolveEnvironmentRequestDto> for ResolveEnvironmentRequest {
    fn from(value: ResolveEnvironmentRequestDto) -> Self {
        Self {
            game_root: value.game_root.map(Into::into),
            documents_dir: value.documents_dir.map(Into::into),
            launcher_db: value.launcher_db.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DetectedEnvironmentDto {
    #[ts(optional)]
    pub game_root: Option<String>,
    #[ts(optional)]
    pub documents_dir: Option<String>,
    #[ts(optional)]
    pub launcher_db: Option<String>,
    pub steam_libraries: Vec<String>,
}

impl From<DetectedEnvironment> for DetectedEnvironmentDto {
    fn from(value: DetectedEnvironment) -> Self {
        Self {
            game_root: value
                .game_root
                .map(|path| path.to_string_lossy().into_owned()),
            documents_dir: value
                .documents_dir
                .map(|path| path.to_string_lossy().into_owned()),
            launcher_db: value
                .launcher_db
                .map(|path| path.to_string_lossy().into_owned()),
            steam_libraries: value
                .steam_libraries
                .into_iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentDto {
    pub game_root: String,
    pub documents_dir: String,
    pub launcher_db: String,
    pub steam_libraries: Vec<String>,
}

impl From<ResolvedEnvironment> for EnvironmentDto {
    fn from(value: ResolvedEnvironment) -> Self {
        Self {
            game_root: value.game_root.to_string_lossy().into_owned(),
            documents_dir: value.documents_dir.to_string_lossy().into_owned(),
            launcher_db: value.launcher_db.to_string_lossy().into_owned(),
            steam_libraries: value
                .steam_libraries
                .into_iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapDataDto {
    pub environment: DetectedEnvironmentDto,
    #[ts(optional)]
    pub environment_error: Option<crate::error::AppError>,
    pub output_directory: String,
}
