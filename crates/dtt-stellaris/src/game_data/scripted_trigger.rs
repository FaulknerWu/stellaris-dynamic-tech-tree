use std::collections::{HashMap, HashSet};

use crate::clausewitz::ClausewitzDocument;
use crate::clausewitz::script::{
    Script, ScriptError, ScriptNote, ScriptNoteKind, object_to_script,
};
use crate::clausewitz::value::{CwEntry, CwObject};
use crate::error::Result;
use crate::load_order::Manifest;

use super::diagnostic::{
    GameDataCategory, GameDataDiagnostic, GameDataDiagnosticKind, Ingested, definition_diagnostic,
    overwrite_diagnostic, parse_diagnostic,
};
use super::inline_script::{InlineScripts, expand_inline_scripts};
use super::{manifest_txt_files, read_bytes};

#[derive(Debug, Default, Clone)]
pub(crate) struct ScriptedTriggers {
    pub definitions: HashMap<String, Script>,
    pub names: HashSet<String>,
}

pub(crate) fn ingest_scripted_triggers(
    manifest: &Manifest,
    inline_scripts: &InlineScripts,
) -> Result<Ingested<ScriptedTriggers>> {
    let mut diagnostics = Vec::new();
    let mut scripted_triggers = ScriptedTriggers::default();
    for file in manifest_txt_files(manifest, "common/scripted_triggers")? {
        let bytes = read_bytes(&file.path)?;
        match ClausewitzDocument::parse(&bytes) {
            Ok(document) => collect_definitions(
                &CwObject::from_utf8(&document.root()),
                file.provenance(),
                inline_scripts,
                &mut scripted_triggers,
                &mut diagnostics,
            ),
            Err(error) => diagnostics.push(parse_diagnostic(
                file.provenance(),
                GameDataCategory::ScriptedTrigger,
                &error,
            )),
        }
    }

    Ok(Ingested::new(scripted_triggers, diagnostics))
}

fn collect_definitions(
    root: &CwObject,
    source: String,
    inline_scripts: &InlineScripts,
    scripted_triggers: &mut ScriptedTriggers,
    diagnostics: &mut Vec<GameDataDiagnostic>,
) {
    for entry in &root.entries {
        let CwEntry::Field(field) = entry else {
            continue;
        };
        if field.key.starts_with('@') {
            continue;
        }
        scripted_triggers.names.insert(field.key.clone());
        let Some(object) = CwObject::as_object_value(&field.value) else {
            diagnostics.push(definition_diagnostic(
                &source,
                GameDataCategory::ScriptedTrigger,
                &field.key,
                format!("脚本触发器 `{}` 的值不是对象", field.key),
            ));
            continue;
        };
        let expanded =
            expand_inline_scripts(object, inline_scripts, &source, &field.key, diagnostics);
        match object_to_script(&expanded) {
            Ok(converted) => {
                diagnostics.extend(script_notes(
                    &source,
                    GameDataCategory::ScriptedTrigger,
                    Some(&field.key),
                    converted.notes,
                ));
                if scripted_triggers
                    .definitions
                    .insert(field.key.clone(), converted.script)
                    .is_some()
                {
                    diagnostics.push(overwrite_diagnostic(
                        &source,
                        GameDataCategory::ScriptedTrigger,
                        &field.key,
                        format!(
                            "脚本触发器 `{}` 被后续数据源覆盖，最终采用 `{source}`",
                            field.key
                        ),
                    ));
                }
            }
            Err(error) => diagnostics.push(definition_diagnostic(
                &source,
                GameDataCategory::ScriptedTrigger,
                &field.key,
                condition_error_message(error, &field.key),
            )),
        }
    }
}

pub(super) fn script_notes(
    source: &str,
    category: GameDataCategory,
    subject: Option<&str>,
    notes: Vec<ScriptNote>,
) -> Vec<GameDataDiagnostic> {
    notes
        .into_iter()
        .map(|note| GameDataDiagnostic {
            source: source.to_string(),
            category,
            subject: subject.map(str::to_string).or(Some(note.trigger.clone())),
            byte_offset: None,
            kind: match note.kind {
                ScriptNoteKind::MalformedArgument | ScriptNoteKind::StructuredArgument => {
                    GameDataDiagnosticKind::UnsupportedCondition
                }
                ScriptNoteKind::DuplicateParameter => GameDataDiagnosticKind::DuplicateParameter,
            },
            message: note.message,
        })
        .collect()
}

pub(super) fn condition_error_message(error: ScriptError, subject: &str) -> String {
    match error {
        ScriptError::IsolatedBranch => {
            format!("`{subject}` 的条件含有孤立的 else_if 或 else，无法可靠转换")
        }
        ScriptError::InvalidValue => {
            format!("`{subject}` 的条件含有无法转换的结构")
        }
    }
}
