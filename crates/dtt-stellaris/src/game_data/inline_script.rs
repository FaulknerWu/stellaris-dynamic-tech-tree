use crate::game_data::DefinitionIssue;
use std::collections::{HashMap, HashSet};

use crate::clausewitz::ClausewitzDocument;
use crate::clausewitz::value::{CwEntry, CwField, CwObject, CwValue, apply_placeholders_to_object};
use crate::error::Result;
use crate::load_order::Manifest;

use super::diagnostic::{
    GameDataCategory, GameDataDiagnostic, Ingested, definition_diagnostic, parse_diagnostic,
};
use super::{manifest_txt_files, read_bytes};

const INLINE_SCRIPT_DIR: &str = "common/inline_scripts";

#[derive(Debug, Default, Clone)]
pub(crate) struct InlineScripts {
    scripts: HashMap<String, CwObject>,
}

impl InlineScripts {
    pub(crate) fn get(&self, name: &str) -> Option<&CwObject> {
        self.scripts.get(&normalise_script_key(name))
    }
}

pub(crate) fn ingest_inline_scripts(manifest: &Manifest) -> Result<Ingested<InlineScripts>> {
    let mut scripts = InlineScripts::default();
    let mut diagnostics = Vec::new();
    for file in manifest_txt_files(manifest, INLINE_SCRIPT_DIR)? {
        let bytes = read_bytes(&file.path)?;
        match ClausewitzDocument::parse(&bytes) {
            Ok(document) => {
                let key = inline_script_key(&file.relative_path);
                let object = CwObject::from_utf8(&document.root());
                if scripts.scripts.insert(key.clone(), object).is_some() {
                    diagnostics.push(definition_diagnostic(
                        file.provenance(),
                        GameDataCategory::InlineScript,
                        &key,
                        DefinitionIssue::Overwritten { previous: None },
                    ));
                }
            }
            Err(error) => diagnostics.push(parse_diagnostic(
                file.provenance(),
                GameDataCategory::InlineScript,
                &error,
            )),
        }
    }
    Ok(Ingested::new(scripts, diagnostics))
}

pub(crate) fn expand_inline_scripts(
    object: &CwObject,
    scripts: &InlineScripts,
    source: &str,
    subject: &str,
    diagnostics: &mut Vec<GameDataDiagnostic>,
) -> CwObject {
    expand_object(
        object,
        scripts,
        source,
        subject,
        diagnostics,
        &mut HashSet::new(),
    )
}

fn expand_object(
    object: &CwObject,
    scripts: &InlineScripts,
    source: &str,
    subject: &str,
    diagnostics: &mut Vec<GameDataDiagnostic>,
    stack: &mut HashSet<String>,
) -> CwObject {
    let mut entries = Vec::new();
    for entry in &object.entries {
        match entry {
            CwEntry::Field(field) if field.key == "inline_script" => {
                entries.extend(expand_field(
                    field,
                    scripts,
                    source,
                    subject,
                    diagnostics,
                    stack,
                ));
            }
            CwEntry::Field(field) => entries.push(CwEntry::Field(CwField {
                key: field.key.clone(),
                operator: field.operator,
                value: expand_value(&field.value, scripts, source, subject, diagnostics, stack),
            })),
            CwEntry::Value(value) => entries.push(CwEntry::Value(expand_value(
                value,
                scripts,
                source,
                subject,
                diagnostics,
                stack,
            ))),
        }
    }
    CwObject { entries }
}

fn expand_field(
    field: &CwField,
    scripts: &InlineScripts,
    source: &str,
    subject: &str,
    diagnostics: &mut Vec<GameDataDiagnostic>,
    stack: &mut HashSet<String>,
) -> Vec<CwEntry> {
    let Some((name, parameters)) = inline_call(&field.value) else {
        diagnostics.push(definition_diagnostic(
            source,
            GameDataCategory::InlineScript,
            subject,
            DefinitionIssue::InvalidInlineCall,
        ));
        return unresolved_inline(field);
    };
    let key = normalise_script_key(&name);
    if !stack.insert(key.clone()) {
        diagnostics.push(definition_diagnostic(
            source,
            GameDataCategory::InlineScript,
            subject,
            DefinitionIssue::InlineCycle {
                script: key.clone(),
            },
        ));
        return unresolved_inline(field);
    }
    let expanded = match scripts.get(&key) {
        Some(body) => {
            let substituted = apply_placeholders_to_object(body, &parameters);
            expand_object(&substituted, scripts, source, subject, diagnostics, stack).entries
        }
        None => {
            diagnostics.push(definition_diagnostic(
                source,
                GameDataCategory::InlineScript,
                subject,
                DefinitionIssue::InlineMissing {
                    script: key.clone(),
                },
            ));
            unresolved_inline(field)
        }
    };
    stack.remove(&key);
    expanded
}

fn unresolved_inline(field: &CwField) -> Vec<CwEntry> {
    vec![CwEntry::Field(CwField {
        key: field.key.clone(),
        operator: field.operator,
        value: CwValue::Malformed,
    })]
}

fn expand_value(
    value: &CwValue,
    scripts: &InlineScripts,
    source: &str,
    subject: &str,
    diagnostics: &mut Vec<GameDataDiagnostic>,
    stack: &mut HashSet<String>,
) -> CwValue {
    match value {
        CwValue::Object(object) => CwValue::Object(expand_object(
            object,
            scripts,
            source,
            subject,
            diagnostics,
            stack,
        )),
        CwValue::Array(values) => CwValue::Array(
            values
                .iter()
                .map(|item| expand_value(item, scripts, source, subject, diagnostics, stack))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn inline_call(value: &CwValue) -> Option<(String, HashMap<String, String>)> {
    match value {
        CwValue::Scalar(name) => Some((name.clone(), HashMap::new())),
        CwValue::Object(object) => {
            let name = object.first_scalar("script")?.to_string();
            let mut parameters = HashMap::new();
            for field in object.fields() {
                if field.key == "script" {
                    continue;
                }
                if let Some(scalar) = field.value.as_scalar() {
                    parameters.insert(field.key.clone(), scalar.to_string());
                }
            }
            Some((name, parameters))
        }
        _ => None,
    }
}

fn inline_script_key(relative_path: &str) -> String {
    let trimmed = relative_path
        .trim_start_matches(INLINE_SCRIPT_DIR)
        .trim_start_matches('/')
        .trim_start_matches('\\');
    normalise_script_key(trimmed)
}

fn normalise_script_key(name: &str) -> String {
    let name = name.replace('\\', "/").trim_matches('"').to_string();
    let name = name.strip_suffix(".txt").unwrap_or(&name);
    name.trim_start_matches("./").to_string()
}
