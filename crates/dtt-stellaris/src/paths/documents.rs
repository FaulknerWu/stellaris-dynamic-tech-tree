use std::path::{Path, PathBuf};

#[cfg(not(windows))]
use super::home_dir;
use super::{
    DiscoveryOptions, LAUNCHER_DB_FILENAME, PARADOX_DIR, SAVE_GAMES_DIR_NAME,
    STELLARIS_INSTALL_DIR_NAME, dedupe_paths, env_path,
};

pub(super) fn find_documents_dir(options: &DiscoveryOptions) -> Option<PathBuf> {
    documents_dir_candidates(options)
        .into_iter()
        .find(|path| is_stellaris_documents_dir(path))
}

fn documents_dir_candidates(options: &DiscoveryOptions) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for preferred in &options.preferred_documents_dirs {
        candidates.push(preferred.clone());
        candidates.push(preferred.join(PARADOX_DIR).join(STELLARIS_INSTALL_DIR_NAME));
        candidates.push(preferred.join(STELLARIS_INSTALL_DIR_NAME));
    }

    #[cfg(windows)]
    {
        for root in windows_documents_roots() {
            candidates.push(root.join(PARADOX_DIR).join(STELLARIS_INSTALL_DIR_NAME));
            candidates.push(root.join(STELLARIS_INSTALL_DIR_NAME));
            candidates.push(root);
        }
    }

    #[cfg(target_os = "macos")]
    {
        for root in macos_documents_roots() {
            candidates.push(root.join(PARADOX_DIR).join(STELLARIS_INSTALL_DIR_NAME));
            candidates.push(root.join(STELLARIS_INSTALL_DIR_NAME));
            candidates.push(root);
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for root in linux_data_roots() {
            candidates.push(root.join(PARADOX_DIR).join(STELLARIS_INSTALL_DIR_NAME));
            candidates.push(root.join(STELLARIS_INSTALL_DIR_NAME));
            candidates.push(root);
        }
    }

    dedupe_paths(candidates)
}

#[cfg(windows)]
fn windows_documents_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(doc_dir) = dirs::document_dir() {
        roots.push(doc_dir);
    }

    roots.extend(windows_registry_documents_roots());

    if let Some(userprofile) = env_path("USERPROFILE") {
        roots.push(userprofile.join("Documents"));
        roots.push(userprofile.join("文档"));
        roots.push(userprofile.join("OneDrive").join("Documents"));
        roots.push(userprofile.join("OneDrive").join("文档"));
    }
    for var in ["OneDrive", "OneDriveConsumer", "OneDriveCommercial"] {
        if let Some(onedrive) = env_path(var) {
            roots.push(onedrive.join("Documents"));
            roots.push(onedrive.join("文档"));
            roots.push(onedrive);
        }
    }
    if let (Some(drive), Some(path)) = (env_path("HOMEDRIVE"), env_path("HOMEPATH")) {
        let base = PathBuf::from(format!("{}{}", drive.display(), path.display()));
        roots.push(base.join("Documents"));
        roots.push(base.join("文档"));
    }
    if let Some(doc) = env_path("DOCUMENTS") {
        roots.push(doc);
    }

    dedupe_paths(roots)
}

#[cfg(windows)]
fn windows_registry_documents_roots() -> Vec<PathBuf> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};

    let mut roots = Vec::new();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if let Ok(key) = hkcu.open_subkey_with_flags(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders",
        KEY_READ,
    ) && let Ok(val) = key.get_value::<String, _>("Personal")
    {
        roots.push(super::expand_win_env_vars(&val));
        if let Ok(val) = key.get_value::<String, _>("{F42EE2D3-909F-4907-8871-4022E96C7F97}") {
            roots.push(super::expand_win_env_vars(&val));
        }
    }

    if let Ok(key) = hkcu.open_subkey_with_flags(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders",
        KEY_READ,
    ) && let Ok(val) = key.get_value::<String, _>("Personal")
    {
        roots.push(super::expand_win_env_vars(&val));
    }

    if let Ok(key) = hkcu.open_subkey_with_flags(r"Software\Microsoft\OneDrive", KEY_READ)
        && let Ok(val) = key.get_value::<String, _>("UserFolder")
    {
        let path = super::expand_win_env_vars(&val);
        roots.push(path.join("Documents"));
        roots.push(path.join("文档"));
        roots.push(path);
    }

    if let Ok(accounts_key) =
        hkcu.open_subkey_with_flags(r"Software\Microsoft\OneDrive\Accounts", KEY_READ)
    {
        for subkey_name in accounts_key.enum_keys().filter_map(std::result::Result::ok) {
            if let Ok(account) = accounts_key.open_subkey_with_flags(&subkey_name, KEY_READ)
                && let Ok(val) = account.get_value::<String, _>("UserFolder")
            {
                let path = super::expand_win_env_vars(&val);
                roots.push(path.join("Documents"));
                roots.push(path.join("文档"));
                roots.push(path);
            }
        }
    }

    roots
}

#[cfg(target_os = "macos")]
fn macos_documents_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(doc_dir) = dirs::document_dir() {
        roots.push(doc_dir);
    }
    if let Some(home) = home_dir() {
        roots.push(home.join("Documents"));
        roots.push(home.join("Library").join("Application Support"));
    }
    dedupe_paths(roots)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn linux_data_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(data_dir) = dirs::data_dir() {
        roots.push(data_dir);
    }
    if let Some(doc_dir) = dirs::document_dir() {
        roots.push(doc_dir);
    }
    if let Some(xdg_data_home) = env_path("XDG_DATA_HOME") {
        roots.push(xdg_data_home);
    }
    if let Some(home) = home_dir() {
        roots.push(home.join(".local").join("share"));
        roots.push(home.join("Documents"));
        roots.push(
            home.join(".var")
                .join("app")
                .join("com.valvesoftware.Steam")
                .join(".local")
                .join("share"),
        );
    }
    dedupe_paths(roots)
}

fn is_stellaris_documents_dir(path: &Path) -> bool {
    path.is_dir()
        && (path.join(LAUNCHER_DB_FILENAME).is_file()
            || path.join(SAVE_GAMES_DIR_NAME).is_dir()
            || path.join("mod").is_dir()
            || path.join("settings.txt").is_file()
            || path.join("dlc_load.json").is_file()
            || path.join("game_data.json").is_file())
}
