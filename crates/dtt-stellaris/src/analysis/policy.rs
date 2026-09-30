use std::collections::HashSet;

use dtt_core::condition::ContextReason;
use dtt_core::condition::{
    ComparisonOperator, Predicate, PredicateArgument, PredicateEvaluation, UnknownConditionReason,
};

use super::AnalysisContext;
use super::scope::{ObjectKind, ObjectRef};

pub(super) fn evaluate(
    predicate: &Predicate,
    context: &AnalysisContext<'_>,
) -> PredicateEvaluation {
    let name = predicate.name.as_str();
    if let PredicateArgument::Malformed = predicate.argument {
        return PredicateEvaluation::unknown(UnknownConditionReason::MalformedArgument(
            name.into(),
        ));
    }
    if is_opening_predicate(name) {
        return opening(predicate, context);
    }
    if is_process_predicate(name) {
        return process(predicate, context);
    }
    match &predicate.argument {
        PredicateArgument::Structured { keys } => {
            PredicateEvaluation::unknown(UnknownConditionReason::StructuredArgument {
                trigger: name.into(),
                keys: keys.clone(),
            })
        }
        _ => context.unknown(name, ContextReason::UnknownTrigger),
    }
}

fn is_opening_predicate(name: &str) -> bool {
    matches!(
        name,
        "always"
            | "has_ethic"
            | "has_civic"
            | "has_valid_civic"
            | "has_authority"
            | "has_origin"
            | "has_government"
            | "is_country_type"
            | "is_ai"
            | "has_trait"
            | "is_archetype"
            | "uses_ship_category"
            | "is_nomadic"
    )
}

fn opening(predicate: &Predicate, context: &AnalysisContext<'_>) -> PredicateEvaluation {
    let name = predicate.name.as_str();
    if !matches!(
        predicate.operator,
        ComparisonOperator::Equal | ComparisonOperator::NotEqual
    ) {
        return PredicateEvaluation::unknown(UnknownConditionReason::Operator {
            trigger: name.into(),
            operator: predicate.operator,
        });
    }
    let PredicateArgument::Scalar(value) = &predicate.argument else {
        return context.unknown(name, ContextReason::ExpectedScalar);
    };
    if value.is_empty() || value.starts_with('@') || value.contains('$') {
        return context.unknown(name, ContextReason::UnboundArgument);
    }
    let flag = matches!(name, "always" | "is_ai" | "is_nomadic");
    if flag && !matches!(value.as_str(), "yes" | "no") {
        return context.unknown(name, ContextReason::InvalidBoolean);
    }
    let negated = (predicate.operator == ComparisonOperator::NotEqual) ^ (flag && value == "no");
    if name == "always" {
        return PredicateEvaluation::fixed(!negated);
    }
    let expected = if matches!(name, "has_trait" | "is_archetype") {
        ObjectKind::Species
    } else {
        ObjectKind::Country
    };
    if context.current.kind() != Some(expected) {
        return context.unknown(
            name,
            ContextReason::TypeMismatch {
                expected: format!("{expected:?}"),
                actual: context.current.kind().map(|kind| format!("{kind:?}")),
            },
        );
    }
    match &context.current {
        ObjectRef::Unresolved { reason, .. } => return context.unknown(name, reason.clone()),
        ObjectRef::Future { .. } => return PredicateEvaluation::deferred(name),
        _ => {}
    }
    let snapshot = context.world.snapshot;
    let found = match name {
        "has_ethic" => snapshot.ethics.as_ref().map(|items| items.contains(value)),
        "has_civic" | "has_valid_civic" => snapshot
            .government
            .civics
            .as_ref()
            .map(|items| items.contains(value)),
        "has_authority" => snapshot
            .government
            .authority
            .as_ref()
            .map(|item| item == value),
        "has_origin" => snapshot
            .government
            .origin
            .as_ref()
            .map(|item| item == value),
        "has_government" => snapshot
            .government
            .government_type
            .as_ref()
            .map(|item| item == value),
        "is_country_type" => snapshot.country_type.as_ref().map(|item| item == value),
        "is_ai" => snapshot.is_ai,
        "is_nomadic" => snapshot.is_nomadic,
        "has_trait" => snapshot
            .founder_species
            .as_ref()
            .and_then(|species| species.traits.as_ref())
            .map(|traits| traits.contains(value)),
        "is_archetype" => snapshot
            .founder_species
            .as_ref()
            .and_then(|species| species.archetype.as_ref())
            .map(|item| item == value),
        "uses_ship_category" => snapshot
            .graphical_culture
            .as_ref()
            .and_then(|culture| context.world.ship_kinds.get(culture))
            .map(|kinds| kinds.contains(value)),
        _ => None,
    };
    match found {
        Some(found) => PredicateEvaluation::fixed(found ^ negated),
        None => context.unknown(name, ContextReason::MissingIdentity),
    }
}

fn is_process_predicate(name: &str) -> bool {
    matches!(
        name,
        "days_passed"
            | "has_communications"
            | "has_crisis_level"
            | "has_dna"
            | "has_federation_perk"
            | "has_global_flag"
            | "has_menace_perk"
            | "has_relic"
            | "is_active_resolution"
            | "resource_expenses_compare"
            | "host_has_dlc"
            | "has_dlc"
            | "has_technology"
            | "can_research_technology"
            | "has_ascension_perk"
            | "has_tradition"
            | "has_federation"
            | "is_subject"
            | "exists"
            | "is_situation_type"
            | "has_country_flag"
            | "has_policy_flag"
    )
}

fn process(predicate: &Predicate, context: &AnalysisContext<'_>) -> PredicateEvaluation {
    let name = predicate.name.as_str();
    let expected = match name {
        "has_federation_perk" => Some(ObjectKind::Federation),
        "is_situation_type" => Some(ObjectKind::Situation),
        "days_passed"
        | "has_global_flag"
        | "is_active_resolution"
        | "host_has_dlc"
        | "has_dlc"
        | "exists" => None,
        _ => Some(ObjectKind::Country),
    };
    if let Some(expected) = expected
        && context.current.kind() != Some(expected)
    {
        return context.unknown(
            name,
            ContextReason::TypeMismatch {
                expected: format!("{expected:?}"),
                actual: context.current.kind().map(|kind| format!("{kind:?}")),
            },
        );
    }
    if let ObjectRef::Unresolved { reason, .. } = &context.current {
        return context.unknown(name, reason.clone());
    }
    let valid_argument = match &predicate.argument {
        PredicateArgument::Parameters(fields)
            if matches!(name, "has_dna" | "resource_expenses_compare") =>
        {
            valid_process_parameters(name, fields)
        }
        PredicateArgument::Scalar(value)
            if !matches!(name, "has_dna" | "resource_expenses_compare") =>
        {
            bound_scalar(value)
                && (!matches!(name, "days_passed" | "has_crisis_level")
                    || value.parse::<f64>().is_ok_and(f64::is_finite))
                && (!matches!(name, "has_federation" | "is_subject")
                    || matches!(value.as_str(), "yes" | "no"))
        }
        _ => false,
    };
    if !valid_argument {
        return context.unknown(name, ContextReason::UnverifiedStructure);
    }
    if !matches!(
        predicate.operator,
        ComparisonOperator::Equal | ComparisonOperator::NotEqual
    ) && !(matches!(name, "days_passed" | "has_crisis_level")
        && matches!(
            predicate.operator,
            ComparisonOperator::LessThan
                | ComparisonOperator::LessThanOrEqual
                | ComparisonOperator::GreaterThan
                | ComparisonOperator::GreaterThanOrEqual
        ))
    {
        return PredicateEvaluation::unknown(UnknownConditionReason::Operator {
            trigger: name.into(),
            operator: predicate.operator,
        });
    }
    PredicateEvaluation::deferred(name)
}

fn bound_scalar(value: &str) -> bool {
    !value.trim().is_empty() && !value.starts_with('@') && !value.contains('$')
}

fn valid_process_parameters(
    name: &str,
    fields: &[dtt_core::condition::PredicateParameter],
) -> bool {
    let required: &[&str] = match name {
        "has_dna" => &["ship_category"],
        "resource_expenses_compare" => &["resource", "value"],
        _ => return false,
    };
    if !required
        .iter()
        .all(|name| fields.iter().any(|field| field.name == *name))
    {
        return false;
    }
    let mut seen = HashSet::new();
    fields.iter().all(|field| {
        if !seen.insert(&field.name) || !bound_scalar(&field.value) {
            return false;
        }
        match (name, field.name.as_str()) {
            ("has_dna", "ship_category" | "rarity")
            | ("resource_expenses_compare", "resource" | "category") => {
                field.operator == ComparisonOperator::Equal
            }
            ("resource_expenses_compare", "value") => {
                field.value.parse::<f64>().is_ok_and(f64::is_finite)
                    && matches!(
                        field.operator,
                        ComparisonOperator::Equal
                            | ComparisonOperator::NotEqual
                            | ComparisonOperator::LessThan
                            | ComparisonOperator::LessThanOrEqual
                            | ComparisonOperator::GreaterThan
                            | ComparisonOperator::GreaterThanOrEqual
                    )
            }
            _ => false,
        }
    })
}
