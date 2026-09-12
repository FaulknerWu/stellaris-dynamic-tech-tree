use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;

mod descriptor;
mod files;
mod launcher;

pub(crate) use files::{ManifestFile, manifest_direct_files, manifest_files};

const BASE_GAME_NAME: &str = "<base game>";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub(crate) sources: Vec<SourceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SourceEntry {
    pub index: u32,
    pub name: String,
    pub root: PathBuf,
    pub replace_paths: Vec<String>,
}

impl Manifest {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    pub(crate) fn is_shadowed_by_replace(&self, rel_path: &str, source_index: u32) -> bool {
        let rel = normalise_rel(rel_path);
        self.sources.iter().any(|source| {
            source.index > source_index
                && source
                    .replace_paths
                    .iter()
                    .any(|replace_path| replace_prefixes(replace_path, &rel))
        })
    }
}

pub fn resolve(stellaris_root: &Path, launcher_database: &Path) -> Result<Manifest> {
    let mods = launcher::read(launcher_database)?;
    Ok(build_manifest(stellaris_root, mods))
}

#[derive(Debug, Clone)]
struct ModRef {
    name: String,
    root: PathBuf,
    replace_paths: Vec<String>,
}

fn build_manifest(stellaris_root: &Path, mods: Vec<ModRef>) -> Manifest {
    let mut sources = Vec::with_capacity(mods.len() + 1);
    sources.push(SourceEntry {
        index: 0,
        name: BASE_GAME_NAME.to_string(),
        root: stellaris_root.to_path_buf(),
        replace_paths: Vec::new(),
    });
    for (index, module) in mods.into_iter().enumerate() {
        sources.push(SourceEntry {
            index: (index + 1) as u32,
            name: module.name,
            root: module.root,
            replace_paths: module.replace_paths,
        });
    }
    Manifest { sources }
}

pub(crate) fn normalise_rel(path: &str) -> String {
    let path = path.replace('\\', "/");
    let path = path.trim_start_matches("./");
    path.trim_matches('/').to_string()
}

fn replace_prefixes(replace_path: &str, rel: &str) -> bool {
    let replace_path = normalise_rel(replace_path);
    if replace_path.is_empty() || rel.is_empty() {
        return false;
    }
    rel == replace_path || rel.starts_with(&format!("{replace_path}/"))
}
