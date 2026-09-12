use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::{Manifest, SourceEntry, normalise_rel};
use crate::error::{Error, Result};

#[derive(Debug)]
pub(crate) struct ManifestFile<'a> {
    pub source: &'a SourceEntry,
    pub path: PathBuf,
    pub relative_path: String,
}

impl ManifestFile<'_> {
    pub(crate) fn source_label(&self) -> String {
        format!("#{}:{}", self.source.index, self.source.name)
    }

    pub(crate) fn provenance(&self) -> String {
        format!(
            "#{}:{}:{}",
            self.source.index, self.source.name, self.relative_path
        )
    }
}

pub(crate) fn manifest_files<'a>(
    manifest: &'a Manifest,
    relative_directory: &str,
) -> Result<Vec<ManifestFile<'a>>> {
    let mut files = Vec::new();
    for source in &manifest.sources {
        let mut source_files = files_for_source(source, relative_directory)?;
        source_files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        files.extend(
            source_files
                .into_iter()
                .filter(|file| !manifest.is_shadowed_by_replace(&file.relative_path, source.index)),
        );
    }
    Ok(files)
}

pub(crate) fn manifest_direct_files<'a>(
    manifest: &'a Manifest,
    relative_directory: &str,
) -> Result<Vec<ManifestFile<'a>>> {
    let mut files = Vec::new();
    for source in &manifest.sources {
        let mut source_files = direct_files_for_source(source, relative_directory)?;
        source_files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        files.extend(
            source_files
                .into_iter()
                .filter(|file| !manifest.is_shadowed_by_replace(&file.relative_path, source.index)),
        );
    }
    Ok(files)
}

fn files_for_source<'a>(
    source: &'a SourceEntry,
    relative_directory: &str,
) -> Result<Vec<ManifestFile<'a>>> {
    let mut paths = Vec::new();
    collect_regular_files(&source.root.join(relative_directory), &mut paths)?;
    Ok(paths
        .into_iter()
        .map(|path| {
            let relative_path = relative_path(&source.root, &path);
            ManifestFile {
                source,
                path,
                relative_path,
            }
        })
        .collect())
}

fn direct_files_for_source<'a>(
    source: &'a SourceEntry,
    relative_directory: &str,
) -> Result<Vec<ManifestFile<'a>>> {
    let directory = source.root.join(relative_directory);
    let metadata = match fs::symlink_metadata(&directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(Error::io(&directory, error)),
    };
    if metadata.file_type().is_symlink() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&directory).map_err(|error| Error::io(&directory, error))?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| Error::io(&directory, error))?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| Error::io(&path, error))?;
        if file_type.is_file() && !file_type.is_symlink() {
            files.push(ManifestFile {
                source,
                relative_path: relative_path(&source.root, &path),
                path,
            });
        }
    }
    Ok(files)
}

fn collect_regular_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let metadata = match fs::symlink_metadata(directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(Error::io(directory, error)),
    };

    if metadata.file_type().is_symlink() {
        return Ok(());
    }

    let entries = fs::read_dir(directory).map_err(|error| Error::io(directory, error))?;
    for entry in entries {
        let entry = entry.map_err(|error| Error::io(directory, error))?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| Error::io(&path, error))?;

        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_regular_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn relative_path(root: &Path, file: &Path) -> String {
    normalise_rel(&file.strip_prefix(root).unwrap_or(file).to_string_lossy())
}
