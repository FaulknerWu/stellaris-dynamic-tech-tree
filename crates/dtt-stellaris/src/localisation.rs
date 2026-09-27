use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::error::Result;
use crate::load_order::{Manifest, ManifestFile, manifest_files};
use dtt_core::technology::{Catalog, Id};

mod parser;

const DESC_SUFFIX: &str = "_desc";
const LOCALISATION_DIR: &str = "localisation";
const REPLACE_DIR_NAME: &str = "replace";
const GENERATED_LOC_PREFIX: &str = "zztechtree";

#[derive(Debug, Default, Clone)]
pub struct LocalisationSet {
    pub descriptions: HashMap<Id, String>,
    pub diagnostics: Vec<LocalisationDiagnostic>,
}

impl LocalisationSet {
    fn ingest_source(&mut self, src: &str, known_ids: &HashSet<Id>) {
        for (key, value) in parser::parse_entries(src) {
            let Some(base_id) = key.strip_suffix(DESC_SUFFIX) else {
                continue;
            };
            let id = Id::from(base_id);
            if known_ids.contains(&id) {
                self.descriptions
                    .insert(id, parser::normalise_whitespace(&value));
            }
        }
    }

    fn apply_swap_description_fallbacks(&mut self, technologies: &Catalog) {
        let mut ordered_technologies: Vec<_> = technologies.values().collect();
        ordered_technologies.sort_by(|left, right| left.id.cmp(&right.id));

        for technology in ordered_technologies {
            let Some(base_description) = self.descriptions.get(&technology.id).cloned() else {
                continue;
            };

            for swap in &technology.technology_swaps {
                self.descriptions
                    .entry(swap.active_id.clone())
                    .or_insert_with(|| base_description.clone());
            }
        }
    }

    fn push_diagnostic(
        &mut self,
        source: &str,
        path: &Path,
        kind: LocalisationFailureKind,
        technical_detail: String,
    ) {
        self.diagnostics.push(LocalisationDiagnostic {
            source: source.to_string(),
            path: path.to_string_lossy().into_owned(),
            kind,
            technical_detail,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalisationDiagnostic {
    pub source: String,
    pub path: String,
    pub kind: LocalisationFailureKind,
    pub technical_detail: String,
}

fn localisation_ids_from_technologies(technologies: &Catalog) -> HashSet<Id> {
    let mut ids = HashSet::new();
    for technology in technologies.values() {
        ids.insert(technology.id.clone());
        for swap in &technology.technology_swaps {
            ids.insert(swap.active_id.clone());
        }
    }
    ids
}

pub fn ingest(
    manifest: &Manifest,
    language: &str,
    technologies: &Catalog,
) -> Result<LocalisationSet> {
    let known_ids = localisation_ids_from_technologies(technologies);
    let mut set = LocalisationSet::default();
    let files = manifest_files(manifest, LOCALISATION_DIR)?;

    ingest_phase(
        &mut set,
        &files,
        language,
        &known_ids,
        LocalisationPhase::Normal,
    );
    ingest_phase(
        &mut set,
        &files,
        language,
        &known_ids,
        LocalisationPhase::Replace,
    );
    set.apply_swap_description_fallbacks(technologies);
    Ok(set)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalisationPhase {
    Normal,
    Replace,
}

fn ingest_phase(
    set: &mut LocalisationSet,
    files: &[ManifestFile<'_>],
    language: &str,
    known_ids: &HashSet<Id>,
    phase: LocalisationPhase,
) {
    for file in files {
        if !matches_localisation_file(file, language, phase) {
            continue;
        }

        let source_label = file.source_label();
        let diagnostic_path = Path::new(&file.relative_path);

        match fs::read(&file.path) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(src) => set.ingest_source(&src, known_ids),
                Err(error) => set.push_diagnostic(
                    &source_label,
                    diagnostic_path,
                    LocalisationFailureKind::InvalidUtf8,
                    error.to_string(),
                ),
            },
            Err(error) => set.push_diagnostic(
                &source_label,
                diagnostic_path,
                LocalisationFailureKind::ReadFailed,
                error.to_string(),
            ),
        }
    }
}

fn matches_localisation_file(
    file: &ManifestFile<'_>,
    language: &str,
    phase: LocalisationPhase,
) -> bool {
    phase_matches_replace_dir(phase, is_replace_relative_path(&file.relative_path))
        && is_language_yml(&file.path, language)
        && !is_generated_dtt_file(&file.path)
}

fn is_replace_relative_path(relative_path: &str) -> bool {
    relative_path
        .split('/')
        .any(|segment| segment == REPLACE_DIR_NAME)
}

fn phase_matches_replace_dir(phase: LocalisationPhase, in_replace_dir: bool) -> bool {
    match phase {
        LocalisationPhase::Normal => !in_replace_dir,
        LocalisationPhase::Replace => in_replace_dir,
    }
}

fn is_generated_dtt_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(GENERATED_LOC_PREFIX))
}

fn is_language_yml(path: &Path, language: &str) -> bool {
    path.extension().is_some_and(|ext| ext == "yml")
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name == format!("l_{language}.yml") || name.ends_with(&format!("_l_{language}.yml"))
            })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalisationFailureKind {
    InvalidUtf8,
    ReadFailed,
}
