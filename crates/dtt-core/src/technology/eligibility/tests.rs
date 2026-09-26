use super::*;
use crate::condition::{Condition, UnknownConditionReason};
use crate::technology::{Definition, SwapVariant};
use crate::test_support::{Context, catalog, compiled, definition, ids, leaf};

fn eligible(definitions: Vec<Definition>) -> EligibilityReport {
    evaluate_eligibility(
        &catalog(definitions),
        &Context,
        UnknownStrategy::IncludeFlagged,
    )
    .unwrap()
}

#[test]
fn missing_prerequisites_remove_transitive_dependents_regardless_of_id_order() {
    let mut first = definition("z_first");
    first.prerequisites.all_of = ids(&["missing"]);
    let mut second = definition("a_second");
    second.prerequisites.all_of = ids(&["z_first"]);
    let mut third = definition("b_third");
    third.prerequisites.all_of = ids(&["a_second"]);
    let result = eligible(vec![third, definition("independent"), first, second]);
    assert_eq!(result.eligible, ids(&["independent"]));
    assert_eq!(
        result.excluded_prereq,
        vec![
            ("a_second".into(), ids(&["z_first"])),
            ("b_third".into(), ids(&["a_second"])),
            ("z_first".into(), ids(&["missing"])),
        ]
    );
}

#[test]
fn each_or_group_requires_one_survivor_in_addition_to_all_required_prerequisites() {
    let mut target = definition("target");
    target.prerequisites.all_of = ids(&["required"]);
    target.prerequisites.any_of_groups = vec![ids(&["missing", "left"]), ids(&["right", "absent"])];
    let full = vec![
        target.clone(),
        definition("required"),
        definition("left"),
        definition("right"),
    ];
    assert_eq!(
        eligible(full.clone()).eligible,
        ids(&["left", "required", "right", "target"])
    );
    for missing in ["required", "left", "right"] {
        let result = eligible(
            full.iter()
                .filter(|item| item.id.as_str() != missing)
                .cloned()
                .collect(),
        );
        assert!(!result.eligible.contains(&target.id), "missing {missing}");
        assert_eq!(result.excluded_prereq.len(), 1);
        assert_eq!(result.excluded_prereq[0].0, target.id);
    }
}

#[test]
fn losing_the_last_or_candidate_propagates_exclusion() {
    let mut failed = definition("z_failed");
    failed.prerequisites.all_of = ids(&["missing"]);
    let mut target = definition("a_target");
    target.prerequisites.any_of_groups = vec![ids(&["z_failed", "absent"])];
    let result = eligible(vec![target, failed]);
    assert!(result.eligible.is_empty());
    assert_eq!(result.excluded_prereq.len(), 2);
}

#[test]
fn unknown_strategies_do_not_treat_valid_future_conditions_as_unsupported() {
    let mut future = definition("future");
    future.potential = Some(compiled(leaf("future")));
    for strategy in [
        UnknownStrategy::IncludeFlagged,
        UnknownStrategy::ExcludeStrict,
        UnknownStrategy::Error,
    ] {
        let result = evaluate_eligibility(&catalog([future.clone()]), &Context, strategy).unwrap();
        assert_eq!(result.eligible, ids(&["future"]));
        assert_eq!(result.uncertain, ids(&["future"]));
        assert!(result.unknown_triggers.is_empty());
        assert_eq!(result.deferred_triggers[0].occurrences, 1);
    }
    let mut unknown = definition("unknown");
    unknown.potential = Some(compiled(Condition::Unknown(
        UnknownConditionReason::Trigger("unsupported".into()),
    )));
    let catalog = catalog([unknown]);
    let included =
        evaluate_eligibility(&catalog, &Context, UnknownStrategy::IncludeFlagged).unwrap();
    assert_eq!(included.eligible, ids(&["unknown"]));
    assert_eq!(included.uncertain, ids(&["unknown"]));
    let excluded =
        evaluate_eligibility(&catalog, &Context, UnknownStrategy::ExcludeStrict).unwrap();
    assert!(excluded.eligible.is_empty());
    assert_eq!(excluded.excluded_identity, ids(&["unknown"]));
    assert!(matches!(
        evaluate_eligibility(&catalog, &Context, UnknownStrategy::Error),
        Err(Error::RuleEngine(_))
    ));
}

#[test]
fn prerequisite_exclusion_removes_uncertain_ids_but_preserves_diagnostics() {
    let mut target = definition("target");
    target.potential = Some(compiled(leaf("future")));
    target.prerequisites.all_of = ids(&["missing"]);
    let result = eligible(vec![target]);
    assert!(result.eligible.is_empty());
    assert!(result.uncertain.is_empty());
    assert_eq!(result.deferred_triggers[0].tech_ids, ids(&["target"]));
}

#[test]
fn closed_cycles_remain_candidates_until_an_external_requirement_fails() {
    let mut a = definition("a");
    let mut b = definition("b");
    a.prerequisites.all_of = ids(&["b"]);
    b.prerequisites.all_of = ids(&["a"]);
    assert_eq!(
        eligible(vec![a.clone(), b.clone()]).eligible,
        ids(&["a", "b"])
    );
    a.prerequisites.all_of.push("missing".into());
    assert!(eligible(vec![a, b]).eligible.is_empty());
}

#[test]
fn real_technology_ids_override_swap_aliases_after_catalog_updates() {
    let mut base = definition("base");
    base.technology_swaps.push(SwapVariant {
        active_id: "alias".into(),
        trigger: compiled(leaf("true")),
        area: None,
    });
    let mut child = definition("child");
    child.prerequisites.all_of = ids(&["alias"]);
    let mut catalog = catalog([base, child]);
    assert_eq!(
        evaluate_eligibility(&catalog, &Context, UnknownStrategy::IncludeFlagged)
            .unwrap()
            .eligible,
        ids(&["base", "child"])
    );
    let mut actual = definition("alias");
    actual.potential = Some(compiled(leaf("false")));
    catalog.insert(actual);
    let result = evaluate_eligibility(&catalog, &Context, UnknownStrategy::IncludeFlagged).unwrap();
    assert_eq!(result.eligible, ids(&["base"]));
    assert_eq!(
        result.excluded_prereq,
        vec![("child".into(), ids(&["alias"]))]
    );
}
