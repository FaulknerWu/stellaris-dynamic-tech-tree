use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::condition::{
    DeferredTriggerOccurrence, EvaluationContext, TruthValue, UnknownTriggerOccurrence,
};
use crate::technology::{Area, Catalog, Definition, Id};
use crate::{Error, Result};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize)]
pub struct SwapEntry {
    pub base_id: Id,
    pub active_id: Id,
    pub area: Area,
    pub outcome: SwapOutcome,
    pub unknown_triggers: Vec<UnknownTriggerOccurrence>,
    pub deferred_triggers: Vec<DeferredTriggerOccurrence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SwapOutcome {
    NoMatch,
    Matched,
    Uncertain,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwapUnknownStrategy {
    #[default]
    KeepBase,
    Error,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct SwapResolution {
    pub display_id: HashMap<Id, Id>,
    pub display_area: HashMap<Id, Area>,
    pub entries: Vec<SwapEntry>,
}

pub fn resolve_swaps(
    catalog: &Catalog,
    eligible: &[Id],
    context: &impl EvaluationContext,
    unknown_strategy: SwapUnknownStrategy,
) -> Result<SwapResolution> {
    let mut res = SwapResolution::default();

    let mut ordered: Vec<&Id> = eligible.iter().collect();
    ordered.sort();

    for base_id in ordered {
        let definition = catalog.get(base_id).ok_or_else(|| {
            Error::Invariant(format!(
                "eligible tech `{base_id}` not found in technology catalog"
            ))
        })?;
        if definition.technology_swaps.is_empty() {
            continue;
        }

        let entry = resolve_one(definition, context, unknown_strategy)?;

        if entry.active_id != *base_id {
            res.display_id
                .insert(base_id.clone(), entry.active_id.clone());
        }
        if entry.area != definition.area {
            res.display_area.insert(base_id.clone(), entry.area);
        }
        res.entries.push(entry);
    }

    validate_unique_display_ids(eligible, &res)?;

    Ok(res)
}

fn validate_unique_display_ids(eligible: &[Id], resolution: &SwapResolution) -> Result<()> {
    let mut owners: HashMap<Id, Id> = HashMap::new();
    for base_id in eligible {
        let active_id = resolution.display_of(base_id);
        if let Some(existing_base_id) = owners.insert(active_id.clone(), base_id.clone()) {
            return Err(Error::RuleEngine(format!(
                "techs `{existing_base_id}` and `{base_id}` both resolve to display id `{active_id}`"
            )));
        }
    }
    Ok(())
}

fn resolve_one(
    def: &Definition,
    context: &impl EvaluationContext,
    unknown_strategy: SwapUnknownStrategy,
) -> Result<SwapEntry> {
    let base_id = &def.id;
    let mut unknown_triggers = BTreeMap::new();
    let mut deferred_triggers: BTreeMap<String, usize> = BTreeMap::new();

    for (idx, variant) in def.technology_swaps.iter().enumerate() {
        let evaluation = variant.trigger.evaluate_detailed(context);
        for diagnostic in evaluation.unknown_triggers {
            *unknown_triggers.entry(diagnostic.reason).or_default() += diagnostic.occurrences;
        }
        for diagnostic in evaluation.deferred_triggers {
            *deferred_triggers.entry(diagnostic.name).or_default() += diagnostic.occurrences;
        }
        if evaluation.bounds.guaranteed == TruthValue::True {
            return Ok(SwapEntry {
                base_id: base_id.clone(),
                active_id: variant.active_id.clone(),
                area: variant.area.unwrap_or(def.area),
                outcome: SwapOutcome::Matched,
                unknown_triggers: collect_unknown_triggers(unknown_triggers),
                deferred_triggers: collect_deferred_triggers(deferred_triggers),
            });
        }
        if evaluation.bounds.possible != TruthValue::False {
            if matches!(unknown_strategy, SwapUnknownStrategy::Error) {
                return Err(Error::RuleEngine(format!(
                    "科技 `{base_id}` 的 technology_swap[{idx}] 无法根据开局身份确定是否匹配"
                )));
            }
            return Ok(SwapEntry {
                base_id: base_id.clone(),
                active_id: base_id.clone(),
                area: def.area,
                outcome: SwapOutcome::Uncertain,
                unknown_triggers: collect_unknown_triggers(unknown_triggers),
                deferred_triggers: collect_deferred_triggers(deferred_triggers),
            });
        }
    }

    Ok(SwapEntry {
        base_id: base_id.clone(),
        active_id: base_id.clone(),
        area: def.area,
        outcome: SwapOutcome::NoMatch,
        unknown_triggers: collect_unknown_triggers(unknown_triggers),
        deferred_triggers: collect_deferred_triggers(deferred_triggers),
    })
}

fn collect_unknown_triggers(
    unknown_triggers: BTreeMap<crate::condition::UnknownConditionReason, usize>,
) -> Vec<UnknownTriggerOccurrence> {
    unknown_triggers
        .into_iter()
        .map(|(reason, occurrences)| UnknownTriggerOccurrence {
            reason,
            occurrences,
        })
        .collect()
}

fn collect_deferred_triggers(
    deferred_triggers: BTreeMap<String, usize>,
) -> Vec<DeferredTriggerOccurrence> {
    deferred_triggers
        .into_iter()
        .map(|(name, occurrences)| DeferredTriggerOccurrence { name, occurrences })
        .collect()
}

impl SwapResolution {
    pub fn display_of(&self, base_id: &Id) -> Id {
        self.display_id
            .get(base_id)
            .cloned()
            .unwrap_or_else(|| base_id.clone())
    }
}
