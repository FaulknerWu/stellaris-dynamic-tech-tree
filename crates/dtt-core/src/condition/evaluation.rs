use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    Condition, ConditionBranch, EvaluationContext, PredicateEvaluation, TruthBounds, TruthValue,
    UnknownConditionReason,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct TriggerEvaluation {
    pub bounds: TruthBounds,
    pub unknown_triggers: Vec<UnknownTriggerOccurrence>,
    pub deferred_triggers: Vec<DeferredTriggerOccurrence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnknownTriggerOccurrence {
    pub reason: UnknownConditionReason,
    pub occurrences: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredTriggerOccurrence {
    pub name: String,
    pub occurrences: usize,
}

#[derive(Default)]
struct Diagnostics {
    unknown: BTreeMap<UnknownConditionReason, usize>,
    deferred: BTreeMap<String, usize>,
}

impl Diagnostics {
    fn record(&mut self, evaluation: PredicateEvaluation) -> TruthBounds {
        if let Some(reason) = evaluation.unknown {
            *self.unknown.entry(reason).or_default() += 1;
        }
        if let Some(name) = evaluation.deferred {
            *self.deferred.entry(name).or_default() += 1;
        }
        evaluation.bounds
    }
}

pub(super) fn evaluate_detailed(
    condition: &Condition,
    context: &impl EvaluationContext,
) -> TriggerEvaluation {
    let mut diagnostics = Diagnostics::default();
    let bounds = eval(condition, context, &mut diagnostics);
    TriggerEvaluation {
        bounds,
        unknown_triggers: diagnostics
            .unknown
            .into_iter()
            .map(|(reason, occurrences)| UnknownTriggerOccurrence {
                reason,
                occurrences,
            })
            .collect(),
        deferred_triggers: diagnostics
            .deferred
            .into_iter()
            .map(|(name, occurrences)| DeferredTriggerOccurrence { name, occurrences })
            .collect(),
    }
}

fn eval(
    condition: &Condition,
    context: &impl EvaluationContext,
    diagnostics: &mut Diagnostics,
) -> TruthBounds {
    match condition {
        Condition::All(items) => {
            let mut result = TruthBounds::fixed(true);
            for item in items {
                result = and(result, eval(item, context, diagnostics));
                if result.possible == TruthValue::False {
                    break;
                }
            }
            result
        }
        Condition::Any(items) => {
            let mut result = TruthBounds::fixed(false);
            for item in items {
                result = or(result, eval(item, context, diagnostics));
                if result.guaranteed == TruthValue::True {
                    break;
                }
            }
            result
        }
        Condition::Not(inner) => negate(eval(inner, context, diagnostics)),
        Condition::If { branches, fallback } => {
            eval_branches(branches, fallback.as_deref(), context, diagnostics)
        }
        Condition::Scope { reference, inner } => {
            let scope = context.scope(reference);
            let presence = diagnostics.record(scope.presence);
            if presence.possible == TruthValue::False {
                return TruthBounds::fixed(scope.optional);
            }
            let body = eval(inner, &scope.context, diagnostics);
            if scope.optional {
                choose(presence, body, TruthBounds::fixed(true))
            } else {
                and(presence, body)
            }
        }
        Condition::AnyObject { collection, inner } => {
            let scope = context.any_object(collection);
            let presence = diagnostics.record(scope.presence);
            if presence.possible == TruthValue::False {
                return presence;
            }
            and(presence, eval(inner, &scope.context, diagnostics))
        }
        Condition::Scripted { name, inner } => eval(inner, &context.scripted(name), diagnostics),
        Condition::Predicate(predicate) => diagnostics.record(context.predicate(predicate)),
        Condition::Unknown(reason) => {
            diagnostics.record(PredicateEvaluation::unknown(reason.clone()))
        }
    }
}

fn eval_branches(
    branches: &[ConditionBranch],
    fallback: Option<&Condition>,
    context: &impl EvaluationContext,
    diagnostics: &mut Diagnostics,
) -> TruthBounds {
    let Some((branch, rest)) = branches.split_first() else {
        return fallback
            .map(|body| eval(body, context, diagnostics))
            .unwrap_or(TruthBounds::fixed(true));
    };
    let limit = eval(&branch.limit, context, diagnostics);
    if limit.guaranteed == TruthValue::True {
        return eval(&branch.body, context, diagnostics);
    }
    if limit.possible == TruthValue::False {
        return eval_branches(rest, fallback, context, diagnostics);
    }
    let body = eval(&branch.body, context, diagnostics);
    let remaining = eval_branches(rest, fallback, context, diagnostics);
    choose(limit, body, remaining)
}

fn truth_and(left: TruthValue, right: TruthValue) -> TruthValue {
    match (left, right) {
        (TruthValue::False, _) | (_, TruthValue::False) => TruthValue::False,
        (TruthValue::True, TruthValue::True) => TruthValue::True,
        _ => TruthValue::Unknown,
    }
}

fn truth_or(left: TruthValue, right: TruthValue) -> TruthValue {
    !truth_and(!left, !right)
}

fn and(left: TruthBounds, right: TruthBounds) -> TruthBounds {
    TruthBounds {
        guaranteed: truth_and(left.guaranteed, right.guaranteed),
        possible: truth_and(left.possible, right.possible),
    }
}

fn or(left: TruthBounds, right: TruthBounds) -> TruthBounds {
    TruthBounds {
        guaranteed: truth_or(left.guaranteed, right.guaranteed),
        possible: truth_or(left.possible, right.possible),
    }
}

fn negate(bounds: TruthBounds) -> TruthBounds {
    TruthBounds {
        guaranteed: !bounds.possible,
        possible: !bounds.guaranteed,
    }
}

fn choose(limit: TruthBounds, yes: TruthBounds, no: TruthBounds) -> TruthBounds {
    TruthBounds {
        possible: truth_or(
            truth_or(
                truth_and(limit.possible, yes.possible),
                truth_and(!limit.guaranteed, no.possible),
            ),
            truth_and(yes.possible, no.possible),
        ),
        guaranteed: truth_and(
            truth_and(
                truth_or(!limit.possible, yes.guaranteed),
                truth_or(limit.guaranteed, no.guaranteed),
            ),
            truth_or(yes.guaranteed, no.guaranteed),
        ),
    }
}
