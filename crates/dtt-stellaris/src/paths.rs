use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

mod documents;
mod steam;

pub const STELLARIS_STEAM_APP_ID: &str = "281990";

pub const STELLARIS_INSTALL_DIR_NAME: &str = "Stellaris";

pub const LAUNCHER_DB_FILENAME: &str = "launcher-v2.sqlite";

pub const SAVE_GAMES_DIR_NAME: &str = "save games";

const STEAMAPPS_DIR: &str = "steamapps";
const COMMON_DIR: &str = "common";
const PARADOX_DIR: &str = "Paradox Interactive";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveryOptions {
    pub preferred_game_roots: Vec<PathBuf>,
    pub preferred_documents_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveredPaths {
    pub game_root: Option<PathBuf>,
    pub documents_dir: Option<PathBuf>,
    pub launcher_db: Option<PathBuf>,
    pub steam_libraries: Vec<PathBuf>,
}

pub fn is_game_root(path: &Path) -> bool {
    path.is_dir() && path.join("common").is_dir() && path.join("localisation").is_dir()
}

pub fn discover(options: &DiscoveryOptions) -> DiscoveredPaths {
    let steam_libraries = steam::discover_libraries();
    let game_root = steam::find_game_root(options, &steam_libraries);
    let documents_dir = documents::find_documents_dir(options);
    let launcher_db = documents_dir
        .as_ref()
        .map(|dir| dir.join(LAUNCHER_DB_FILENAME))
        .filter(|path| path.is_file());

    DiscoveredPaths {
        game_root,
        documents_dir,
        launcher_db,
        steam_libraries,
    }
}

pub fn default_launcher_db(documents_dir: &Path) -> PathBuf {
    documents_dir.join(LAUNCHER_DB_FILENAME)
}

pub fn default_save_games_dir(documents_dir: &Path) -> PathBuf {
    documents_dir.join(SAVE_GAMES_DIR_NAME)
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(not(windows))]
fn home_dir() -> Option<PathBuf> {
    env_path("HOME").or_else(|| env_path("USERPROFILE"))
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for path in paths {
        let mut key = path.to_string_lossy().replace('\\', "/");
        if cfg!(windows) {
            key = key.to_lowercase();
        }
        if seen.insert(key) {
            out.push(path);
        }
    }
    out
}

#[cfg(windows)]
pub(super) fn expand_win_env_vars(raw: &str) -> PathBuf {
    let mut result = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut var_name = String::new();
            let mut matched = false;
            while let Some(&next_c) = chars.peek() {
                chars.next();
                if next_c == '%' {
                    matched = true;
                    break;
                }
                var_name.push(next_c);
            }
            if matched {
                if let Ok(val) = env::var(&var_name) {
                    result.push_str(&val);
                } else {
                    result.push('%');
                    result.push_str(&var_name);
                    result.push('%');
                }
            } else {
                result.push('%');
                result.push_str(&var_name);
            }
        } else {
            result.push(c);
        }
    }
    PathBuf::from(result)
}
