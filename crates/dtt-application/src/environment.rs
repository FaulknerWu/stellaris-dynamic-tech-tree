use crate::error::SettingsIssue;
use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::generation::{GenerationSettings, ResolveGenerationSettingsRequest};

pub use dtt_stellaris::paths::{
    DiscoveredPaths as DetectedEnvironment, DiscoveryOptions as DetectEnvironmentRequest,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolveEnvironmentRequest {
    pub game_root: Option<PathBuf>,
    pub documents_dir: Option<PathBuf>,
    pub launcher_db: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedEnvironment {
    pub game_root: PathBuf,
    pub documents_dir: PathBuf,
    pub launcher_db: PathBuf,
    pub steam_libraries: Vec<PathBuf>,
}

pub fn detect_environment(request: &DetectEnvironmentRequest) -> DetectedEnvironment {
    dtt_stellaris::paths::discover(request)
}

pub fn resolve_environment(request: &ResolveEnvironmentRequest) -> Result<ResolvedEnvironment> {
    validate_explicit_path(
        request.game_root.as_deref(),
        dtt_stellaris::paths::is_game_root,
        "game_root",
    )?;
    validate_explicit_path(
        request.documents_dir.as_deref(),
        Path::is_dir,
        "documents_dir",
    )?;
    validate_explicit_path(request.launcher_db.as_deref(), Path::is_file, "launcher_db")?;

    let detected = merge_detected(
        request.game_root.clone(),
        request.documents_dir.clone(),
        request.launcher_db.clone(),
    );
    let game_root = detected.game_root.ok_or_else(|| {
        Error::Settings(SettingsIssue::Missing {
            field: "stellaris_root",
        })
    })?;
    let documents_dir = detected.documents_dir.ok_or_else(|| {
        Error::Settings(SettingsIssue::Missing {
            field: "documents_dir",
        })
    })?;
    let launcher_db = detected.launcher_db.ok_or_else(|| {
        Error::Settings(SettingsIssue::Missing {
            field: "launcher_db",
        })
    })?;

    Ok(ResolvedEnvironment {
        game_root,
        documents_dir,
        launcher_db,
        steam_libraries: detected.steam_libraries,
    })
}

pub fn output_directory() -> Result<PathBuf> {
    let executable = env::current_exe().map_err(|error| Error::ExecutablePath(error))?;
    executable
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| Error::ExecutableParentMissing)
}

pub fn resolve_generation_settings(
    request: &ResolveGenerationSettingsRequest,
) -> Result<GenerationSettings> {
    let detected = merge_detected(
        request.stellaris_root.clone(),
        request.documents_dir.clone(),
        request.launcher_db.clone(),
    );
    let stellaris_root = detected.game_root.ok_or_else(|| {
        Error::Settings(SettingsIssue::Missing {
            field: "stellaris_root",
        })
    })?;
    let launcher_db = detected.launcher_db.ok_or_else(|| {
        Error::Settings(SettingsIssue::Missing {
            field: "launcher_db",
        })
    })?;
    let settings = GenerationSettings {
        stellaris_root: stellaris_root.to_string_lossy().into_owned(),
        launcher_db: launcher_db.to_string_lossy().into_owned(),
        languages: normalise_languages(&request.languages)?,
        unknown_strategy: request.unknown_strategy,
        swap_unknown_strategy: request.swap_unknown_strategy,
    };
    settings.validate()?;
    Ok(settings)
}

fn merge_detected(
    game_root: Option<PathBuf>,
    documents_dir: Option<PathBuf>,
    launcher_db: Option<PathBuf>,
) -> DetectedEnvironment {
    let detected = detect_environment(&DetectEnvironmentRequest {
        preferred_game_roots: game_root.iter().cloned().collect(),
        preferred_documents_dirs: documents_dir.iter().cloned().collect(),
    });
    let documents_dir = documents_dir.or(detected.documents_dir);
    let launcher_db = launcher_db
        .or_else(|| {
            documents_dir
                .as_deref()
                .map(dtt_stellaris::paths::default_launcher_db)
                .filter(|path| path.is_file())
        })
        .or(detected.launcher_db);
    DetectedEnvironment {
        game_root: game_root.or(detected.game_root),
        documents_dir,
        launcher_db,
        steam_libraries: detected.steam_libraries,
    }
}

fn validate_explicit_path(
    path: Option<&Path>,
    predicate: impl FnOnce(&Path) -> bool,
    field: &'static str,
) -> Result<()> {
    if let Some(path) = path
        && !predicate(path)
    {
        return Err(Error::Settings(SettingsIssue::InvalidPath {
            field,
            path: path.to_path_buf(),
        }));
    }
    Ok(())
}

fn normalise_languages(
    languages: &[dtt_stellaris::output::GameLanguage],
) -> Result<Vec<dtt_stellaris::output::GameLanguage>> {
    let mut normalised = Vec::new();
    for language in languages {
        if !normalised.contains(language) {
            normalised.push(*language);
        }
    }
    if normalised.is_empty() {
        return Err(Error::Settings(SettingsIssue::NoOutputLanguage));
    }
    Ok(normalised)
}
