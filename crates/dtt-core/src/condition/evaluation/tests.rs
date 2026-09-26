use super::*;
use crate::test_support::{Context, compiled, leaf};

fn evaluate(condition: Condition) -> TriggerEvaluation {
    compiled(condition).evaluate_detailed(&Context)
}

fn unknown() -> Condition {
    Condition::Unknown(UnknownConditionReason::Trigger("unsupported".into()))
}

#[test]
fn boolean_operators_obey_the_complete_three_value_truth_tables() {
    use TruthValue::{False as F, True as T, Unknown as U};
    let operands = [leaf("false"), leaf("true"), unknown()];
    let and_table = [[F, F, F], [F, T, U], [F, U, U]];
    let or_table = [[F, T, U], [T, T, T], [U, T, U]];
    for (i, left) in operands.iter().enumerate() {
        for (j, right) in operands.iter().enumerate() {
            for (condition, expected) in [
                (
                    Condition::All(vec![left.clone(), right.clone()]),
                    and_table[i][j],
                ),
                (
                    Condition::Any(vec![left.clone(), right.clone()]),
                    or_table[i][j],
                ),
            ] {
                assert_eq!(
                    evaluate(condition).bounds,
                    TruthBounds {
                        guaranteed: expected,
                        possible: expected
                    },
                    "operands {i}, {j}"
                );
            }
        }
    }
}

#[test]
fn empty_all_is_true_and_empty_any_is_false() {
    assert_eq!(
        evaluate(Condition::All(vec![])).bounds,
        TruthBounds::fixed(true)
    );
    assert_eq!(
        evaluate(Condition::Any(vec![])).bounds,
        TruthBounds::fixed(false)
    );
}

#[test]
fn negating_future_conditions_preserves_future_possibility() {
    let result = evaluate(Condition::Not(Box::new(leaf("future"))));
    assert_eq!(result.bounds, TruthBounds::deferred());
    assert!(result.unknown_triggers.is_empty());
    assert_eq!(
        result.deferred_triggers,
        vec![DeferredTriggerOccurrence {
            name: "future".into(),
            occurrences: 1,
        }]
    );
    assert_eq!(
        evaluate(Condition::Not(Box::new(unknown()))).bounds,
        TruthBounds::unknown()
    );
}

#[test]
fn decisive_operands_do_not_report_unvisited_unknown_triggers() {
    for condition in [
        Condition::All(vec![leaf("false"), unknown()]),
        Condition::Any(vec![leaf("true"), unknown()]),
    ] {
        assert!(evaluate(condition).unknown_triggers.is_empty());
    }
    let result = evaluate(Condition::All(vec![unknown(), unknown(), leaf("false")]));
    assert_eq!(result.bounds, TruthBounds::fixed(false));
    assert_eq!(
        result.unknown_triggers,
        vec![UnknownTriggerOccurrence {
            reason: UnknownConditionReason::Trigger("unsupported".into()),
            occurrences: 2,
        }]
    );
}

#[test]
fn uncertain_branch_limits_preserve_agreement_between_bodies() {
    for limit in [unknown(), leaf("future")] {
        for value in [false, true] {
            let body = leaf(if value { "true" } else { "false" });
            let result = evaluate(Condition::If {
                branches: vec![ConditionBranch {
                    limit: limit.clone(),
                    body: body.clone(),
                }],
                fallback: Some(Box::new(body)),
            });
            assert_eq!(result.bounds, TruthBounds::fixed(value));
        }
    }
}

#[test]
fn first_matching_branch_wins_and_missing_fallback_is_true() {
    let result = evaluate(Condition::If {
        branches: vec![ConditionBranch {
            limit: leaf("true"),
            body: leaf("false"),
        }],
        fallback: Some(Box::new(unknown())),
    });
    assert_eq!(result.bounds, TruthBounds::fixed(false));
    assert!(result.unknown_triggers.is_empty());
    assert_eq!(
        evaluate(Condition::If {
            branches: vec![ConditionBranch {
                limit: leaf("false"),
                body: unknown()
            }],
            fallback: None,
        })
        .bounds,
        TruthBounds::fixed(true)
    );
}

#[test]
fn absent_scopes_distinguish_optional_access_from_existential_queries() {
    for (reference, expected) in [("absent", false), ("absent?", true)] {
        let result = evaluate(Condition::Scope {
            reference: reference.into(),
            inner: Box::new(unknown()),
        });
        assert_eq!(result.bounds, TruthBounds::fixed(expected));
        assert!(result.unknown_triggers.is_empty());
    }
    let result = evaluate(Condition::AnyObject {
        collection: "empty".into(),
        inner: Box::new(unknown()),
    });
    assert_eq!(result.bounds, TruthBounds::fixed(false));
    assert!(result.unknown_triggers.is_empty());
}
