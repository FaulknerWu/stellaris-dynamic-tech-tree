use std::collections::HashSet;

use dtt_core::condition::ComparisonOperator;

use super::value::{CwEntry, CwField, CwObject, CwValue};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Script {
    All(Vec<Script>),
    Any(Vec<Script>),
    Not(Box<Script>),
    If {
        branches: Vec<IfBranch>,
        fallback: Option<Box<Script>>,
    },
    Scope {
        reference: String,
        inner: Box<Script>,
    },
    AnyObject {
        collection: String,
        inner: Box<Script>,
    },
    Call {
        name: String,
        operator: ComparisonOperator,
        argument: ScriptArgument,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IfBranch {
    pub limit: Script,
    pub body: Script,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ScriptArgument {
    None,
    Scalar(String),
    Parameters(Vec<ScriptParameter>),
    Structured { keys: Vec<String> },
    Malformed,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScriptParameter {
    pub name: String,
    pub operator: ComparisonOperator,
    pub value: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ScriptConversion {
    pub script: Script,
    pub notes: Vec<ScriptNote>,
}

#[derive(Debug, Clone)]
pub(crate) struct ScriptNote {
    pub kind: ScriptNoteKind,
    pub trigger: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScriptNoteKind {
    MalformedArgument,
    StructuredArgument,
    DuplicateParameter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScriptError {
    IsolatedBranch,
    InvalidValue,
}

struct ConversionCtx {
    notes: Vec<ScriptNote>,
}

pub(crate) fn object_to_script(object: &CwObject) -> Result<ScriptConversion, ScriptError> {
    let mut ctx = ConversionCtx { notes: Vec::new() };
    let script = fold_all(object, &mut ctx)?;
    Ok(ScriptConversion {
        script,
        notes: ctx.notes,
    })
}

fn fold_all(object: &CwObject, ctx: &mut ConversionCtx) -> Result<Script, ScriptError> {
    let scripts = fold_sequence(object, ctx)?;
    Ok(match scripts.len() {
        0 => Script::All(Vec::new()),
        1 => scripts.into_iter().next().expect("len == 1"),
        _ => Script::All(scripts),
    })
}

fn fold_any(object: &CwObject, ctx: &mut ConversionCtx) -> Result<Script, ScriptError> {
    Ok(Script::Any(fold_sequence(object, ctx)?))
}

fn fold_sequence(object: &CwObject, ctx: &mut ConversionCtx) -> Result<Vec<Script>, ScriptError> {
    fold_entries(&object.entries, ctx)
}

fn fold_entries(entries: &[CwEntry], ctx: &mut ConversionCtx) -> Result<Vec<Script>, ScriptError> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < entries.len() {
        match &entries[index] {
            CwEntry::Field(field) => {
                if is_equal(field) {
                    match field.key.as_str() {
                        "if" => {
                            let (script, consumed) = fold_if_chain(&entries[index..], ctx)?;
                            out.push(script);
                            index += consumed;
                            continue;
                        }
                        "else_if" | "else" => return Err(ScriptError::IsolatedBranch),
                        _ => {}
                    }
                }
                if let Some(script) = field_to_script(field, ctx)? {
                    out.push(script);
                }
            }
            CwEntry::Value(value) => {
                if let Some(script) = remainder_to_script(value, ctx)? {
                    out.push(script);
                }
            }
        }
        index += 1;
    }
    Ok(out)
}

fn fold_if_chain(
    entries: &[CwEntry],
    ctx: &mut ConversionCtx,
) -> Result<(Script, usize), ScriptError> {
    let mut branches = Vec::new();
    let mut fallback = None;
    let mut consumed = 0;

    while consumed < entries.len() {
        let CwEntry::Field(field) = &entries[consumed] else {
            break;
        };
        if !is_equal(field) {
            break;
        }
        match field.key.as_str() {
            "if" if consumed > 0 => break,
            "if" | "else_if" => {
                let object =
                    CwObject::as_object_value(&field.value).ok_or(ScriptError::InvalidValue)?;
                branches.push(if_branch(object, ctx)?);
                consumed += 1;
            }
            "else" => {
                let object =
                    CwObject::as_object_value(&field.value).ok_or(ScriptError::InvalidValue)?;
                fallback = Some(Box::new(fold_all(object, ctx)?));
                consumed += 1;
                break;
            }
            _ => break,
        }
    }

    Ok((Script::If { branches, fallback }, consumed.max(1)))
}

fn if_branch(object: &CwObject, ctx: &mut ConversionCtx) -> Result<IfBranch, ScriptError> {
    let mut limit = None;
    let mut body_entries = Vec::new();
    for entry in &object.entries {
        match entry {
            CwEntry::Field(field) if is_equal(field) && field.key == "limit" => {
                let inner =
                    CwObject::as_object_value(&field.value).ok_or(ScriptError::InvalidValue)?;
                limit = Some(fold_all(inner, ctx)?);
            }
            entry => body_entries.push(entry.clone()),
        }
    }

    let limit = limit.unwrap_or_else(|| Script::All(Vec::new()));
    let body_scripts = fold_entries(&body_entries, ctx)?;
    let body = match body_scripts.len() {
        0 => Script::All(Vec::new()),
        1 => body_scripts.into_iter().next().expect("len == 1"),
        _ => Script::All(body_scripts),
    };
    Ok(IfBranch { limit, body })
}

fn is_equal(field: &CwField) -> bool {
    field.operator == ComparisonOperator::Equal
}

fn field_to_script(
    field: &CwField,
    ctx: &mut ConversionCtx,
) -> Result<Option<Script>, ScriptError> {
    if !is_equal(field) {
        return Ok(Some(call_script(field, ctx)));
    }
    match field.key.as_str() {
        "AND" | "hidden_trigger" | "custom_tooltip" | "limit" => {
            if let Some(object) = CwObject::as_object_value(&field.value) {
                Ok(Some(fold_all(object, ctx)?))
            } else {
                Ok(Some(call_script(field, ctx)))
            }
        }
        "OR" => {
            if let Some(object) = CwObject::as_object_value(&field.value) {
                Ok(Some(fold_any(object, ctx)?))
            } else {
                Ok(Some(call_script(field, ctx)))
            }
        }
        "NOT" | "NAND" => {
            if let Some(object) = CwObject::as_object_value(&field.value) {
                Ok(Some(Script::Not(Box::new(fold_all(object, ctx)?))))
            } else {
                Ok(Some(call_script(field, ctx)))
            }
        }
        "NOR" => {
            if let Some(object) = CwObject::as_object_value(&field.value) {
                Ok(Some(Script::Not(Box::new(fold_any(object, ctx)?))))
            } else {
                Ok(Some(call_script(field, ctx)))
            }
        }
        "if" | "else_if" => {
            let object =
                CwObject::as_object_value(&field.value).ok_or(ScriptError::InvalidValue)?;
            Ok(Some(Script::If {
                branches: vec![if_branch(object, ctx)?],
                fallback: None,
            }))
        }
        "else" => Err(ScriptError::IsolatedBranch),
        "fail_text" | "success_text" | "text" if matches!(field.value, CwValue::Scalar(_)) => {
            Ok(None)
        }
        key if key.starts_with("any_") => {
            if let Some(object) = CwObject::as_object_value(&field.value) {
                Ok(Some(Script::AnyObject {
                    collection: key.to_string(),
                    inner: Box::new(fold_all(object, ctx)?),
                }))
            } else {
                Ok(Some(call_script(field, ctx)))
            }
        }
        key if is_scope_reference(key) => {
            if let Some(object) = CwObject::as_object_value(&field.value) {
                Ok(Some(Script::Scope {
                    reference: key.to_string(),
                    inner: Box::new(fold_all(object, ctx)?),
                }))
            } else {
                Ok(Some(call_script(field, ctx)))
            }
        }
        _ => Ok(Some(call_script(field, ctx))),
    }
}

fn remainder_to_script(
    value: &CwValue,
    ctx: &mut ConversionCtx,
) -> Result<Option<Script>, ScriptError> {
    match value {
        CwValue::Scalar(name) if name == "optimize_memory" => Ok(None),
        CwValue::Scalar(name) => Ok(Some(Script::Call {
            name: name.clone(),
            operator: ComparisonOperator::Equal,
            argument: ScriptArgument::None,
        })),
        CwValue::Object(object) => fold_all(object, ctx).map(Some),
        CwValue::Array(values) => {
            let mut scripts = Vec::new();
            for item in values {
                if let Some(script) = remainder_to_script(item, ctx)? {
                    scripts.push(script);
                }
            }
            Ok(Some(Script::All(scripts)))
        }
        CwValue::Malformed => Err(ScriptError::InvalidValue),
    }
}

fn call_script(field: &CwField, ctx: &mut ConversionCtx) -> Script {
    let argument = script_argument(&field.key, &field.value, ctx);
    Script::Call {
        name: field.key.clone(),
        operator: field.operator,
        argument,
    }
}

fn script_argument(trigger: &str, value: &CwValue, ctx: &mut ConversionCtx) -> ScriptArgument {
    match value {
        CwValue::Scalar(scalar) => ScriptArgument::Scalar(scalar.clone()),
        CwValue::Malformed => {
            ctx.notes
                .push(script_note(ScriptNoteKind::MalformedArgument, trigger));
            ScriptArgument::Malformed
        }
        CwValue::Array(_) => {
            let keys = Vec::new();
            ctx.notes
                .push(script_note(ScriptNoteKind::StructuredArgument, trigger));
            ScriptArgument::Structured { keys }
        }
        CwValue::Object(object) => object_argument(trigger, object, ctx),
    }
}

fn object_argument(trigger: &str, object: &CwObject, ctx: &mut ConversionCtx) -> ScriptArgument {
    if is_collection_iterator(trigger) {
        let keys = object.fields().map(|field| field.key.clone()).collect();
        ctx.notes
            .push(script_note(ScriptNoteKind::StructuredArgument, trigger));
        return ScriptArgument::Structured { keys };
    }
    let mut parameters = Vec::new();
    let mut seen = HashSet::new();
    let mut keys = Vec::new();
    let mut nested = false;

    for entry in &object.entries {
        match entry {
            CwEntry::Field(field) => {
                keys.push(field.key.clone());
                if !seen.insert(field.key.clone()) {
                    ctx.notes
                        .push(script_note(ScriptNoteKind::DuplicateParameter, trigger));
                    return ScriptArgument::Malformed;
                }
                match &field.value {
                    CwValue::Scalar(scalar) => parameters.push(ScriptParameter {
                        name: field.key.clone(),
                        operator: field.operator,
                        value: scalar.clone(),
                    }),
                    _ => nested = true,
                }
            }
            CwEntry::Value(_) => nested = true,
        }
    }

    if nested {
        ctx.notes
            .push(script_note(ScriptNoteKind::StructuredArgument, trigger));
        ScriptArgument::Structured { keys }
    } else {
        ScriptArgument::Parameters(parameters)
    }
}

fn script_note(kind: ScriptNoteKind, trigger: &str) -> ScriptNote {
    ScriptNote {
        kind,
        trigger: trigger.to_string(),
    }
}

fn is_collection_iterator(name: &str) -> bool {
    name.starts_with("any_") || name.starts_with("every_") || name.starts_with("count_")
}

fn is_scope_reference(key: &str) -> bool {
    let key = key.trim_end_matches('?').to_ascii_lowercase();
    matches!(
        key.as_str(),
        "owner_species"
            | "founder_species"
            | "species"
            | "federation"
            | "owner"
            | "this"
            | "root"
            | "capital_scope"
            | "leader"
    ) || key.starts_with("event_target:")
        || key.starts_with("prev")
        || key.starts_with("from")
        || key.contains('.')
}
