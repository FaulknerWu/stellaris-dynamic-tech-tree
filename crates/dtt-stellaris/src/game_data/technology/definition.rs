use std::collections::HashSet;

use dtt_core::condition::CompiledCondition;
use dtt_core::technology::{Area, Definition, Id, Prerequisites, SwapVariant};

use crate::clausewitz::script::{ScriptError, ScriptNote, object_to_script};
use crate::clausewitz::value::{CwEntry, CwObject, CwValue, resolve_at_variable};

use super::super::diagnostic::{GameDataCategory, GameDataDiagnostic, definition_diagnostic};
use super::super::lower::{LowerInputs, lower_script};
use super::super::scripted_trigger::{condition_error_message, script_notes};

const CONSUMED_KEYS: &[&str] = &[
    "area",
    "tier",
    "prerequisites",
    "potential",
    "is_dangerous",
    "is_rare",
    "technology_swap",
    "inline_script",
];

const RECOGNIZED_IGNORED_KEYS: &[&str] = &[
    "start_tech",
    "category",
    "cost",
    "weight",
    "weight_modifier",
    "ai_weight",
    "ai_update_type",
    "modifier",
    "prereqfor_desc",
    "gateway",
    "is_reverse_engineerable",
    "feature_flags",
    "weight_groups",
    "mod_weight_if_group_picked",
    "levels",
    "cost_per_level",
    "starting_potential",
    "is_insight",
    "icon",
    "is_repeatable",
];

pub(super) struct Extracted {
    pub definition: Definition,
    pub unhandled_fields: Vec<String>,
}

pub(super) fn extract(
    name: &str,
    object: &CwObject,
    inputs: &LowerInputs<'_>,
    reachable_scripted: &mut HashSet<String>,
    source: &str,
    diagnostics: &mut Vec<GameDataDiagnostic>,
) -> Option<Extracted> {
    let id = Id::from(name);
    diagnose_fields(name, object, source, diagnostics);

    let Some(area) = first_scalar(object, "area", inputs)
        .as_deref()
        .and_then(parse_area)
    else {
        diagnostics.push(definition_diagnostic(
            source,
            GameDataCategory::Technology,
            name,
            format!("科技 `{name}` 缺少有效的 area 字段"),
        ));
        return None;
    };

    let Some(tier) = number_field(object, "tier", inputs).map(|tier| tier as i32) else {
        diagnostics.push(definition_diagnostic(
            source,
            GameDataCategory::Technology,
            name,
            format!("科技 `{name}` 缺少有效的 tier 字段"),
        ));
        return None;
    };

    let mut conversion_notes = Vec::new();
    let potential = match potential_from(object, inputs, reachable_scripted, &mut conversion_notes)
    {
        Ok(potential) => potential,
        Err(error) => {
            diagnostics.push(definition_diagnostic(
                source,
                GameDataCategory::Technology,
                name,
                condition_error_message(error, name),
            ));
            return None;
        }
    };
    diagnostics.extend(script_notes(
        source,
        GameDataCategory::Technology,
        Some(name),
        conversion_notes,
    ));

    let (technology_swaps, swap_diagnostics) =
        swaps_from(object, inputs, reachable_scripted, source, name);
    diagnostics.extend(swap_diagnostics);

    Some(Extracted {
        definition: Definition {
            id,
            area,
            tier,
            prerequisites: prerequisites_from(object, inputs),
            potential,
            is_dangerous: bool_field(object, "is_dangerous", inputs),
            is_rare: bool_field(object, "is_rare", inputs),
            technology_swaps,
        },
        unhandled_fields: collect_unknown(object),
    })
}

fn parse_area(value: &str) -> Option<Area> {
    match value {
        "physics" => Some(Area::Physics),
        "society" => Some(Area::Society),
        "engineering" => Some(Area::Engineering),
        _ => None,
    }
}

fn resolve_scalar(value: &str, inputs: &LowerInputs<'_>) -> String {
    resolve_at_variable(value, inputs.variables)
}

fn first_scalar(object: &CwObject, name: &str, inputs: &LowerInputs<'_>) -> Option<String> {
    object
        .first_scalar(name)
        .map(|value| resolve_scalar(value, inputs))
}

fn bool_field(object: &CwObject, name: &str, inputs: &LowerInputs<'_>) -> bool {
    matches!(first_scalar(object, name, inputs).as_deref(), Some("yes"))
}

fn number_field(object: &CwObject, name: &str, inputs: &LowerInputs<'_>) -> Option<i64> {
    first_scalar(object, name, inputs)?.parse::<i64>().ok()
}

fn prerequisites_from(object: &CwObject, inputs: &LowerInputs<'_>) -> Prerequisites {
    let Some(field) = object.first_field("prerequisites") else {
        return Prerequisites::default();
    };
    let mut all_of = Vec::new();
    let mut any_of_groups = Vec::new();
    match &field.value {
        CwValue::Object(inner) => {
            collect_prerequisites(inner, inputs, &mut all_of, &mut any_of_groups)
        }
        CwValue::Array(values) => {
            for value in values {
                push_prereq_id(value, inputs, &mut all_of);
            }
        }
        CwValue::Scalar(id) => all_of.push(Id::from(resolve_scalar(id, inputs).as_str())),
        CwValue::Malformed => {}
    }
    Prerequisites {
        all_of,
        any_of_groups,
    }
}

fn collect_prerequisites(
    object: &CwObject,
    inputs: &LowerInputs<'_>,
    all_of: &mut Vec<Id>,
    any_of_groups: &mut Vec<Vec<Id>>,
) {
    for entry in &object.entries {
        match entry {
            CwEntry::Field(field) if field.key == "OR" => {
                let mut group = Vec::new();
                match &field.value {
                    CwValue::Object(inner) => {
                        for nested in &inner.entries {
                            if let CwEntry::Value(value) = nested {
                                push_prereq_id(value, inputs, &mut group);
                            }
                        }
                    }
                    CwValue::Array(values) => {
                        for value in values {
                            push_prereq_id(value, inputs, &mut group);
                        }
                    }
                    other => push_prereq_id(other, inputs, &mut group),
                }
                if !group.is_empty() {
                    any_of_groups.push(group);
                }
            }
            CwEntry::Value(value) => push_prereq_id(value, inputs, all_of),
            CwEntry::Field(_) => {}
        }
    }
}

fn push_prereq_id(value: &CwValue, inputs: &LowerInputs<'_>, dest: &mut Vec<Id>) {
    if let Some(id) = value.as_scalar() {
        dest.push(Id::from(resolve_scalar(id, inputs).as_str()));
    }
}

fn potential_from(
    object: &CwObject,
    inputs: &LowerInputs<'_>,
    reachable_scripted: &mut HashSet<String>,
    notes: &mut Vec<ScriptNote>,
) -> Result<Option<CompiledCondition>, ScriptError> {
    let Some(field) = object.first_field("potential") else {
        return Ok(None);
    };
    let inner = CwObject::as_object_value(&field.value).ok_or(ScriptError::InvalidValue)?;
    let converted = object_to_script(inner)?;
    notes.extend(converted.notes);
    Ok(Some(lower_script(
        &converted.script,
        inputs,
        reachable_scripted,
    )))
}

fn swaps_from(
    object: &CwObject,
    inputs: &LowerInputs<'_>,
    reachable_scripted: &mut HashSet<String>,
    source: &str,
    tech_name: &str,
) -> (Vec<SwapVariant>, Vec<GameDataDiagnostic>) {
    let mut swaps = Vec::new();
    let mut diagnostics = Vec::new();
    let mut declaration_index = 0usize;
    for field in object.fields() {
        if field.key != "technology_swap" {
            continue;
        }
        let index = declaration_index;
        declaration_index += 1;
        let Some(swap) = CwObject::as_object_value(&field.value) else {
            diagnostics.push(definition_diagnostic(
                source,
                GameDataCategory::Technology,
                tech_name,
                format!("科技 `{tech_name}` 的 technology_swap[{index}] 不是对象"),
            ));
            continue;
        };
        let Some(active_id) = first_scalar(swap, "name", inputs).map(Id::from) else {
            diagnostics.push(definition_diagnostic(
                source,
                GameDataCategory::Technology,
                tech_name,
                format!("科技 `{tech_name}` 的 technology_swap[{index}] 缺少 name"),
            ));
            continue;
        };
        let trigger = match swap.first_field("trigger") {
            None => CompiledCondition::always(),
            Some(trigger_field) => {
                let Some(trigger_object) = CwObject::as_object_value(&trigger_field.value) else {
                    diagnostics.push(definition_diagnostic(
                        source,
                        GameDataCategory::Technology,
                        tech_name,
                        format!(
                            "科技 `{tech_name}` 的 technology_swap[{index}] 的 trigger 不是对象"
                        ),
                    ));
                    continue;
                };
                match object_to_script(trigger_object) {
                    Ok(converted) => {
                        diagnostics.extend(script_notes(
                            source,
                            GameDataCategory::Technology,
                            Some(tech_name),
                            converted.notes,
                        ));
                        lower_script(&converted.script, inputs, reachable_scripted)
                    }
                    Err(error) => {
                        diagnostics.push(definition_diagnostic(
                            source,
                            GameDataCategory::Technology,
                            tech_name,
                            condition_error_message(error, tech_name),
                        ));
                        continue;
                    }
                }
            }
        };
        let area = match swap.first_field("area") {
            None => None,
            Some(_) => match first_scalar(swap, "area", inputs)
                .as_deref()
                .and_then(parse_area)
            {
                Some(area) => Some(area),
                None => {
                    diagnostics.push(definition_diagnostic(
                        source,
                        GameDataCategory::Technology,
                        tech_name,
                        format!("科技 `{tech_name}` 的 technology_swap[{index}] 含有无效的 area 字段，跳过该变体"),
                    ));
                    continue;
                }
            },
        };
        swaps.push(SwapVariant {
            active_id,
            trigger,
            area,
        });
    }
    (swaps, diagnostics)
}

fn diagnose_fields(
    name: &str,
    object: &CwObject,
    source: &str,
    diagnostics: &mut Vec<GameDataDiagnostic>,
) {
    let mut seen = HashSet::new();
    let mut reported = HashSet::new();
    for field in object.fields() {
        if CONSUMED_KEYS.contains(&field.key.as_str()) {
            if field.key != "technology_swap"
                && field.key != "inline_script"
                && !seen.insert(field.key.as_str())
                && reported.insert(field.key.as_str())
            {
                diagnostics.push(definition_diagnostic(
                    source,
                    GameDataCategory::Technology,
                    name,
                    format!("科技 `{name}` 重复声明字段 `{}`；当前仅读取第一处，游戏的合并或覆盖语义尚未核实", field.key),
                ));
            }
        } else if !RECOGNIZED_IGNORED_KEYS.contains(&field.key.as_str())
            && let Some(expected) = CONSUMED_KEYS
                .iter()
                .chain(RECOGNIZED_IGNORED_KEYS)
                .find(|expected| expected.eq_ignore_ascii_case(&field.key))
        {
            diagnostics.push(definition_diagnostic(
                source,
                GameDataCategory::Technology,
                name,
                format!("科技 `{name}` 的字段 `{}` 与已识别字段 `{expected}` 大小写不同；当前不自动转换，游戏的大小写规则尚未核实", field.key),
            ));
        }
    }
}

fn collect_unknown(object: &CwObject) -> Vec<String> {
    let mut seen = HashSet::new();
    object
        .fields()
        .filter_map(|field| {
            if CONSUMED_KEYS.contains(&field.key.as_str())
                || RECOGNIZED_IGNORED_KEYS.contains(&field.key.as_str())
            {
                None
            } else if seen.insert(field.key.clone()) {
                Some(field.key.clone())
            } else {
                None
            }
        })
        .collect()
}
