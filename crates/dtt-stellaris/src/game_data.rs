use std::fs;
use std::path::Path;

use dtt_core::condition::GraphicalCultures;

use crate::error::{Error, Result};
use crate::load_order::{Manifest, ManifestFile, manifest_direct_files, manifest_files};

mod diagnostic;
mod graphical_culture;
mod inline_script;
mod lower;
mod scripted_trigger;
mod scripted_variable;
mod technology;

#[cfg(test)]
mod tests;

pub use diagnostic::{
    DefinitionIssue, GameDataCategory, GameDataDiagnostic, GameDataDiagnosticKind, Ingested,
};
pub use technology::{TechnologyIngest, TechnologySourceMetadata};

use graphical_culture::ingest_graphical_cultures;
use inline_script::ingest_inline_scripts;
use lower::LowerInputs;
use scripted_trigger::ingest_scripted_triggers;
use scripted_variable::ingest_scripted_variables;
use technology::ingest_technologies;

#[derive(Debug, Clone)]
pub struct GameData {
    pub technologies: TechnologyIngest,
    pub ship_kinds: GraphicalCultures,
}

pub fn load_game_data(manifest: &Manifest) -> Result<Ingested<GameData>> {
    let mut diagnostics = Vec::new();

    let variables = ingest_scripted_variables(manifest)?;
    diagnostics.extend(variables.diagnostics);

    let inline_scripts = ingest_inline_scripts(manifest)?;
    diagnostics.extend(inline_scripts.diagnostics);

    let scripted = ingest_scripted_triggers(manifest, &inline_scripts.value)?;

    let graphical = ingest_graphical_cultures(manifest)?;
    diagnostics.extend(graphical.diagnostics);

    let inputs = LowerInputs {
        scripted: &scripted.value.definitions,
        scripted_names: &scripted.value.names,
        variables: &variables.value,
    };
    let technologies = ingest_technologies(manifest, &inputs, &inline_scripts.value)?;
    diagnostics.extend(scripted.diagnostics.into_iter().filter(|diagnostic| {
        diagnostic.subject.as_ref().is_none_or(|subject| {
            technologies
                .value
                .reachable_scripted_triggers_by_technology
                .values()
                .any(|reachable| reachable.contains(subject))
        })
    }));
    diagnostics.extend(technologies.diagnostics);

    Ok(Ingested::new(
        GameData {
            technologies: technologies.value,
            ship_kinds: graphical.value,
        },
        diagnostics,
    ))
}

fn manifest_txt_files<'a>(
    manifest: &'a Manifest,
    relative_directory: &str,
) -> Result<Vec<ManifestFile<'a>>> {
    let mut files = manifest_files(manifest, relative_directory)?;
    files.retain(|file| is_txt_file(&file.path));
    Ok(files)
}

fn manifest_direct_txt_files<'a>(
    manifest: &'a Manifest,
    relative_directory: &str,
) -> Result<Vec<ManifestFile<'a>>> {
    let mut files = manifest_direct_files(manifest, relative_directory)?;
    files.retain(|file| is_txt_file(&file.path));
    Ok(files)
}

fn is_txt_file(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "txt")
}

fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|error| Error::io(path, error))
}
