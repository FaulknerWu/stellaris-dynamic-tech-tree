use crate::game_data::DefinitionIssue;
use std::collections::{HashMap, HashSet};

use dtt_core::technology::{Catalog, Id};

use crate::clausewitz::ClausewitzDocument;
use crate::clausewitz::value::{CwEntry, CwObject, CwValue};
use crate::error::Result;
use crate::load_order::Manifest;

use super::diagnostic::{
    GameDataCategory, GameDataDiagnostic, Ingested, definition_diagnostic, overwrite_diagnostic,
    parse_diagnostic,
};
use super::inline_script::{InlineScripts, expand_inline_scripts};
use super::lower::LowerInputs;
use super::{manifest_direct_txt_files, read_bytes};

mod definition;

#[derive(Debug, Clone)]
pub struct TechnologySourceMetadata {
    pub source: String,
    pub unhandled_fields: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct TechnologyIngest {
    pub catalog: Catalog,
    pub metadata: HashMap<Id, TechnologySourceMetadata>,
    pub(crate) reachable_scripted_triggers_by_technology: HashMap<Id, HashSet<String>>,
}

pub(crate) fn ingest_technologies(
    manifest: &Manifest,
    inputs: &LowerInputs<'_>,
    inline_scripts: &InlineScripts,
) -> Result<Ingested<TechnologyIngest>> {
    let mut ingest = TechnologyIngest::default();
    let mut diagnostics = Vec::new();
    for file in manifest_direct_txt_files(manifest, "common/technology")? {
        let bytes = read_bytes(&file.path)?;
        match ClausewitzDocument::parse(&bytes) {
            Ok(document) => collect_technologies(
                &CwObject::from_utf8(&document.root()),
                &file.provenance(),
                inputs,
                inline_scripts,
                &mut ingest,
                &mut diagnostics,
            ),
            Err(error) => diagnostics.push(parse_diagnostic(
                file.provenance(),
                GameDataCategory::Technology,
                &error,
            )),
        }
    }
    Ok(Ingested::new(ingest, diagnostics))
}

fn collect_technologies(
    root: &CwObject,
    source: &str,
    inherited: &LowerInputs<'_>,
    inline_scripts: &InlineScripts,
    ingest: &mut TechnologyIngest,
    diagnostics: &mut Vec<GameDataDiagnostic>,
) {
    let mut file_variables = inherited.variables.clone();
    for entry in &root.entries {
        let CwEntry::Field(field) = entry else {
            continue;
        };
        if let Some(variable_name) = field.key.strip_prefix('@') {
            match &field.value {
                CwValue::Scalar(scalar) => {
                    file_variables.insert(variable_name.to_string(), scalar.clone());
                }
                _ => diagnostics.push(definition_diagnostic(
                    source,
                    GameDataCategory::Technology,
                    variable_name,
                    DefinitionIssue::ExpectedScalar,
                )),
            }
            continue;
        }

        let Some(object) = CwObject::as_object_value(&field.value) else {
            diagnostics.push(definition_diagnostic(
                source,
                GameDataCategory::Technology,
                &field.key,
                DefinitionIssue::ExpectedObject,
            ));
            continue;
        };
        let expanded =
            expand_inline_scripts(object, inline_scripts, source, &field.key, diagnostics);
        let inputs = LowerInputs {
            scripted: inherited.scripted,
            scripted_names: inherited.scripted_names,
            variables: &file_variables,
        };
        let mut reachable_scripted_triggers = HashSet::new();
        if let Some(extracted) = definition::extract(
            &field.key,
            &expanded,
            &inputs,
            &mut reachable_scripted_triggers,
            source,
            diagnostics,
        ) {
            let id = extracted.definition.id.clone();
            if let Some(_old) = ingest.catalog.insert(extracted.definition) {
                let previous = ingest
                    .metadata
                    .get(&id)
                    .map(|metadata| metadata.source.as_str())
                    .unwrap_or("?");
                diagnostics.push(overwrite_diagnostic(
                    source,
                    GameDataCategory::Technology,
                    id.as_str(),
                    DefinitionIssue::Overwritten {
                        previous: Some(previous.to_string()),
                    },
                ));
            }
            ingest.metadata.insert(
                id.clone(),
                TechnologySourceMetadata {
                    source: source.to_string(),
                    unhandled_fields: extracted.unhandled_fields,
                },
            );
            ingest
                .reachable_scripted_triggers_by_technology
                .insert(id, reachable_scripted_triggers);
        }
    }
}
