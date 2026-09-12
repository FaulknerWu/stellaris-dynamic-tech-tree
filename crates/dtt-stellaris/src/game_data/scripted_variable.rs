use std::collections::HashMap;

use crate::clausewitz::{ClausewitzDocument, read_scalar};
use crate::error::Result;
use crate::load_order::Manifest;

use super::diagnostic::{
    GameDataCategory, GameDataDiagnostic, Ingested, definition_diagnostic, parse_diagnostic,
};
use super::{manifest_txt_files, read_bytes};

pub fn ingest_scripted_variables(manifest: &Manifest) -> Result<Ingested<HashMap<String, String>>> {
    let mut variables = HashMap::new();
    let mut diagnostics = Vec::new();
    for file in manifest_txt_files(manifest, "common/scripted_variables")? {
        let bytes = read_bytes(&file.path)?;
        match ClausewitzDocument::parse(&bytes) {
            Ok(document) => {
                collect_variables(
                    document.root(),
                    &file.provenance(),
                    &mut variables,
                    &mut diagnostics,
                );
            }
            Err(error) => diagnostics.push(parse_diagnostic(
                file.provenance(),
                GameDataCategory::ScriptedVariable,
                &error,
            )),
        }
    }
    Ok(Ingested::new(variables, diagnostics))
}

fn collect_variables(
    root: crate::clausewitz::Utf8Object<'_, '_>,
    source: &str,
    variables: &mut HashMap<String, String>,
    diagnostics: &mut Vec<GameDataDiagnostic>,
) {
    for (key, _operator, value) in root.fields() {
        let name = key.read_string();
        let Some(variable_name) = name.strip_prefix('@') else {
            continue;
        };
        match read_scalar(&value) {
            Ok(scalar) => {
                variables.insert(variable_name.to_string(), scalar);
            }
            Err(_) => diagnostics.push(definition_diagnostic(
                source,
                GameDataCategory::ScriptedVariable,
                variable_name,
                format!("脚本变量 `@{variable_name}` 的值不是标量"),
            )),
        }
    }
}
