use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use serde::Serialize;

use crate::condition::TruthValue;
use crate::condition::{
    DeferredTriggerOccurrence, EvaluationContext, UnknownConditionReason, UnknownStrategy,
};
use crate::technology::{Catalog, Id, Prerequisites};
use crate::{Error, Result};

#[derive(Debug, Default, Clone, Serialize)]
pub struct EligibilityReport {
    pub eligible: Vec<Id>,
    pub excluded_identity: Vec<Id>,
    pub excluded_prereq: Vec<(Id, Vec<Id>)>,
    pub uncertain: Vec<Id>,
    pub unknown_triggers: Vec<UnknownTriggerDiagnostic>,
    pub deferred_triggers: Vec<DeferredTriggerDiagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnknownTriggerDiagnostic {
    pub reason: UnknownConditionReason,
    pub occurrences: usize,
    pub tech_ids: Vec<Id>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeferredTriggerDiagnostic {
    pub name: String,
    pub occurrences: usize,
    pub tech_ids: Vec<Id>,
}

pub fn evaluate_eligibility(
    catalog: &Catalog,
    context: &impl EvaluationContext,
    strategy: UnknownStrategy,
) -> Result<EligibilityReport> {
    let order: Vec<Id> = catalog.sorted_ids();
    let prerequisites = catalog.normalised_prerequisites();

    let mut candidates: HashSet<Id> = HashSet::new();
    let mut uncertain: HashSet<Id> = HashSet::new();
    let mut excluded_identity: Vec<Id> = Vec::new();
    let mut unknown_triggers: BTreeMap<UnknownConditionReason, (usize, BTreeSet<Id>)> =
        BTreeMap::new();
    let mut deferred_triggers: BTreeMap<String, (usize, BTreeSet<Id>)> = BTreeMap::new();

    for id in &order {
        let evaluation = catalog
            .get(id)
            .and_then(|technology| technology.potential.as_ref())
            .map(|condition| condition.evaluate_detailed(context));
        let mut guaranteed = TruthValue::True;
        let tri = match evaluation {
            Some(evaluation) => {
                for trigger in evaluation.unknown_triggers {
                    let (occurrences, tech_ids) = unknown_triggers
                        .entry(trigger.reason)
                        .or_insert_with(|| (0, BTreeSet::new()));
                    *occurrences += trigger.occurrences;
                    tech_ids.insert(id.clone());
                }
                for trigger in evaluation.deferred_triggers {
                    accumulate_deferred(&mut deferred_triggers, trigger, id);
                }
                guaranteed = evaluation.bounds.guaranteed;
                evaluation.bounds.possible
            }
            None => TruthValue::True,
        };

        match tri {
            TruthValue::True => {
                candidates.insert(id.clone());
                if guaranteed != TruthValue::True {
                    uncertain.insert(id.clone());
                }
            }
            TruthValue::False => excluded_identity.push(id.clone()),
            TruthValue::Unknown => match strategy {
                UnknownStrategy::IncludeFlagged => {
                    candidates.insert(id.clone());
                    uncertain.insert(id.clone());
                }
                UnknownStrategy::ExcludeStrict => excluded_identity.push(id.clone()),
                UnknownStrategy::Error => {
                    return Err(Error::RuleEngine(format!(
                        "tech `{id}` has an unresolvable `potential` trigger"
                    )));
                }
            },
        }
    }

    let mut dependents: HashMap<Id, Vec<Id>> = HashMap::new();
    for id in &order {
        if let Some(requirements) = prerequisites.get(id) {
            for prerequisite in requirements
                .all_of
                .iter()
                .chain(requirements.any_of_groups.iter().flatten())
            {
                dependents
                    .entry(prerequisite.clone())
                    .or_default()
                    .push(id.clone());
            }
        }
    }

    let mut excluded_prereq: Vec<(Id, Vec<Id>)> = Vec::new();
    let mut queue: VecDeque<Id> = order
        .iter()
        .filter(|id| candidates.contains(*id))
        .filter_map(|id| {
            let requirements = prerequisites
                .get(id)
                .expect("sorted technology id must exist in the catalog");
            missing_prereqs(requirements, &candidates).map(|_| id.clone())
        })
        .collect();

    let mut queued: HashSet<Id> = queue.iter().cloned().collect();
    while let Some(id) = queue.pop_front() {
        queued.remove(&id);
        if !candidates.contains(&id) {
            continue;
        }
        let requirements = prerequisites
            .get(&id)
            .expect("sorted technology id must exist in the catalog");
        let Some(missing) = missing_prereqs(requirements, &candidates) else {
            continue;
        };
        candidates.remove(&id);
        excluded_prereq.push((id.clone(), missing));
        if let Some(downstream) = dependents.get(&id) {
            for dependent in downstream {
                if candidates.contains(dependent) && queued.insert(dependent.clone()) {
                    queue.push_back(dependent.clone());
                }
            }
        }
    }

    let mut uncertain: Vec<Id> = uncertain
        .into_iter()
        .filter(|id| candidates.contains(id))
        .collect();
    uncertain.sort();

    let mut eligible: Vec<Id> = candidates.into_iter().collect();
    eligible.sort();
    excluded_identity.sort();
    excluded_prereq.sort_by(|a, b| a.0.cmp(&b.0));
    let unknown_triggers = unknown_triggers
        .into_iter()
        .map(
            |(reason, (occurrences, tech_ids))| UnknownTriggerDiagnostic {
                reason,
                occurrences,
                tech_ids: tech_ids.into_iter().collect(),
            },
        )
        .collect();
    let deferred_triggers = deferred_triggers
        .into_iter()
        .map(
            |(name, (occurrences, tech_ids))| DeferredTriggerDiagnostic {
                name,
                occurrences,
                tech_ids: tech_ids.into_iter().collect(),
            },
        )
        .collect();

    Ok(EligibilityReport {
        eligible,
        excluded_identity,
        excluded_prereq,
        uncertain,
        unknown_triggers,
        deferred_triggers,
    })
}

fn accumulate_deferred(
    deferred_triggers: &mut BTreeMap<String, (usize, BTreeSet<Id>)>,
    trigger: DeferredTriggerOccurrence,
    technology_id: &Id,
) {
    let (occurrences, tech_ids) = deferred_triggers
        .entry(trigger.name)
        .or_insert_with(|| (0, BTreeSet::new()));
    *occurrences += trigger.occurrences;
    tech_ids.insert(technology_id.clone());
}

fn missing_prereqs(requirements: &Prerequisites, candidates: &HashSet<Id>) -> Option<Vec<Id>> {
    let mut missing = Vec::new();

    for p in &requirements.all_of {
        if !candidates.contains(p) {
            missing.push(p.clone());
        }
    }

    for group in &requirements.any_of_groups {
        if group.iter().all(|m| !candidates.contains(m)) {
            missing.extend(group.iter().cloned());
        }
    }

    if missing.is_empty() {
        None
    } else {
        Some(missing)
    }
}
