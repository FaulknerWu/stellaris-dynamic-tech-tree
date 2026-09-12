use std::fs;
use std::path::{Path, PathBuf};

#[cfg(not(windows))]
use super::home_dir;
use super::{
    COMMON_DIR, DiscoveryOptions, STEAMAPPS_DIR, STELLARIS_INSTALL_DIR_NAME, dedupe_paths, env_path,
};

const LIBRARYFOLDERS_FILE: &str = "libraryfolders.vdf";
const APPMANIFEST_FILE: &str = "appmanifest_281990.acf";

pub(super) fn discover_libraries() -> Vec<PathBuf> {
    let mut found = Vec::new();
    for root in steam_root_candidates() {
        if has_steamapps_dir(&root) {
            found.push(root.clone());
        }

        for library_file in libraryfolders_candidates(&root) {
            let Ok(src) = fs::read_to_string(&library_file) else {
                continue;
            };
            for library in parse_vdf_values(&src, "path") {
                let path = PathBuf::from(library);
                if has_steamapps_dir(&path) {
                    found.push(path);
                }
            }
        }
    }
    dedupe_paths(found)
}

pub(super) fn find_game_root(
    options: &DiscoveryOptions,
    steam_libraries: &[PathBuf],
) -> Option<PathBuf> {
    game_root_candidates(options, steam_libraries)
        .into_iter()
        .find(|path| super::is_game_root(path))
}

fn game_root_candidates(options: &DiscoveryOptions, steam_libraries: &[PathBuf]) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    candidates.extend(options.preferred_game_roots.iter().cloned());

    for library in steam_libraries {
        if let Some(path) = install_dir_from_appmanifest(library) {
            candidates.push(path);
        }
        candidates.push(stellaris_in_library(library));
    }

    dedupe_paths(candidates)
}

fn steam_root_candidates() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    roots.extend(["STEAM_DIR", "STEAM_HOME"].into_iter().filter_map(env_path));

    #[cfg(windows)]
    {
        roots.extend(windows_registry_steam_roots());
        roots.extend(
            ["ProgramFiles(x86)", "ProgramFiles"]
                .into_iter()
                .filter_map(env_path)
                .map(|root| root.join("Steam")),
        );
        if let Some(local_app_data) = env_path("LOCALAPPDATA") {
            roots.push(local_app_data.join("Steam"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = home_dir() {
            roots.push(
                home.join("Library")
                    .join("Application Support")
                    .join("Steam"),
            );
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(home) = home_dir() {
            roots.push(home.join(".steam").join("steam"));
            roots.push(home.join(".steam").join("root"));
            roots.push(home.join(".local").join("share").join("Steam"));
            roots.push(
                home.join(".var")
                    .join("app")
                    .join("com.valvesoftware.Steam")
                    .join(".local")
                    .join("share")
                    .join("Steam"),
            );
        }
    }

    dedupe_paths(roots)
}

#[cfg(windows)]
fn windows_registry_steam_roots() -> Vec<PathBuf> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};

    let mut roots = Vec::new();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey_with_flags(r"Software\Valve\Steam", KEY_READ)
        && let Ok(val) = key.get_value::<String, _>("SteamPath")
    {
        roots.push(super::expand_win_env_vars(&val));
    }

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey_with_flags(r"SOFTWARE\WOW6432Node\Valve\Steam", KEY_READ)
        && let Ok(val) = key.get_value::<String, _>("InstallPath")
    {
        roots.push(super::expand_win_env_vars(&val));
    }
    if let Ok(key) = hklm.open_subkey_with_flags(r"SOFTWARE\Valve\Steam", KEY_READ)
        && let Ok(val) = key.get_value::<String, _>("InstallPath")
    {
        roots.push(super::expand_win_env_vars(&val));
    }

    roots
}

fn libraryfolders_candidates(steam_root: &Path) -> Vec<PathBuf> {
    vec![
        steam_root.join(STEAMAPPS_DIR).join(LIBRARYFOLDERS_FILE),
        steam_root.join("config").join(LIBRARYFOLDERS_FILE),
    ]
}

fn stellaris_in_library(library: &Path) -> PathBuf {
    library
        .join(STEAMAPPS_DIR)
        .join(COMMON_DIR)
        .join(STELLARIS_INSTALL_DIR_NAME)
}

fn install_dir_from_appmanifest(library: &Path) -> Option<PathBuf> {
    let manifest = library.join(STEAMAPPS_DIR).join(APPMANIFEST_FILE);
    let src = fs::read_to_string(manifest).ok()?;
    let install_dir = parse_vdf_values(&src, "installdir")
        .into_iter()
        .next()
        .unwrap_or_else(|| STELLARIS_INSTALL_DIR_NAME.to_string());
    Some(
        library
            .join(STEAMAPPS_DIR)
            .join(COMMON_DIR)
            .join(install_dir),
    )
}

fn has_steamapps_dir(path: &Path) -> bool {
    path.join(STEAMAPPS_DIR).is_dir()
}

fn parse_vdf_values(src: &str, wanted_key: &str) -> Vec<String> {
    let strings = parse_vdf_strings(src);
    let mut values = Vec::new();
    let mut index = 0;
    while index + 1 < strings.len() {
        if strings[index] == wanted_key {
            values.push(strings[index + 1].clone());
            index += 2;
        } else {
            index += 1;
        }
    }
    values
}

fn parse_vdf_strings(src: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '"' {
            continue;
        }

        let mut value = String::new();
        while let Some(inner) = chars.next() {
            match inner {
                '"' => break,
                '\\' => match chars.next() {
                    Some('\\') => value.push('\\'),
                    Some('"') => value.push('"'),
                    Some(other) => {
                        value.push('\\');
                        value.push(other);
                    }
                    None => value.push('\\'),
                },
                other => value.push(other),
            }
        }
        strings.push(value);
    }
    strings
}
