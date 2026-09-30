use crate::condition::{CompiledCondition, Condition, EvaluationContext, TruthValue};

pub(super) fn covers_all(conditions: &[&Condition], context: &impl EvaluationContext) -> bool {
    let mut alternatives = Vec::new();
    for condition in conditions {
        if let Some(residual) = residual(condition, context)
            && add_alternative(residual, &mut alternatives)
        {
            return true;
        }
    }
    false
}

fn add_alternative(condition: Condition, alternatives: &mut Vec<Condition>) -> bool {
    match condition {
        Condition::All(items) if items.is_empty() => true,
        Condition::Any(items) => items
            .into_iter()
            .any(|item| add_alternative(item, alternatives)),
        condition => {
            if alternatives.iter().any(|other| match (&condition, other) {
                (Condition::Not(inner), other) | (other, Condition::Not(inner)) => {
                    inner.as_ref() == other
                }
                _ => false,
            }) {
                return true;
            }
            alternatives.push(condition);
            false
        }
    }
}

fn residual(condition: &Condition, context: &impl EvaluationContext) -> Option<Condition> {
    let evaluation = CompiledCondition {
        condition: condition.clone(),
    }
    .evaluate_detailed(context);
    if evaluation.bounds.guaranteed == TruthValue::True {
        return Some(Condition::All(Vec::new()));
    }
    if evaluation.bounds.possible == TruthValue::False {
        return Some(Condition::Any(Vec::new()));
    }
    if !evaluation.unknown_triggers.is_empty()
        || evaluation.bounds.guaranteed == TruthValue::Unknown
        || evaluation.bounds.possible == TruthValue::Unknown
    {
        return None;
    }
    match condition {
        Condition::All(items) | Condition::Any(items) => {
            let all = matches!(condition, Condition::All(_));
            let mut remaining = Vec::new();
            for item in items {
                let item = residual(item, context)?;
                match item {
                    Condition::All(children) if all => remaining.extend(children),
                    Condition::Any(children) if !all => remaining.extend(children),
                    item => remaining.push(item),
                }
            }
            Some(if remaining.len() == 1 {
                remaining.pop().expect("one residual condition")
            } else if all {
                Condition::All(remaining)
            } else {
                Condition::Any(remaining)
            })
        }
        Condition::Not(inner) => Some(match residual(inner, context)? {
            Condition::Not(inner) => *inner,
            inner => Condition::Not(Box::new(inner)),
        }),
        _ => Some(condition.clone()),
    }
}
