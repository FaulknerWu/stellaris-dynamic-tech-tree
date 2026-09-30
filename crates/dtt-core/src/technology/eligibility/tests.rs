use super::*;
use crate::condition::{Condition, UnknownConditionReason};
use crate::technology::{Definition, SwapVariant};
use crate::test_support::{Context, catalog, compiled, definition, ids, leaf};

#[test]
fn uncertain_prerequisites_propagate_through_multiple_dependency_levels() {
    for condition in [
        leaf("future"),
        Condition::Unknown(UnknownConditionReason::Trigger("unsupported".into())),
    ] {
        let mut root = definition("z_root");
        root.potential = Some(compiled(condition));
        let mut child = definition("a_child");
        child.prerequisites.all_of = ids(&["z_root"]);
        let mut grandchild = definition("b_grandchild");
        grandchild.prerequisites.all_of = ids(&["a_child"]);
        let result = eligible(vec![grandchild, child, root, definition("independent")]);
        assert_eq!(
            result.uncertain,
            ids(&["a_child", "b_grandchild", "z_root"])
        );
        assert_eq!(result.eligible.len(), 4);
    }
}

#[test]
fn each_or_group_needs_a_certain_alternative_to_avoid_uncertainty() {
    let mut future = definition("future");
    future.potential = Some(compiled(leaf("future")));
    let mut certain_choice = definition("certain_choice");
    certain_choice.prerequisites.any_of_groups = vec![ids(&["future", "certain"])];
    let mut uncertain_group = definition("uncertain_group");
    uncertain_group.prerequisites.any_of_groups =
        vec![ids(&["future", "certain"]), ids(&["future", "absent"])];
    let mut uncertain_required = definition("uncertain_required");
    uncertain_required.prerequisites.all_of = ids(&["future"]);
    uncertain_required.prerequisites.any_of_groups = vec![ids(&["future", "certain"])];
    let result = eligible(vec![
        future,
        definition("certain"),
        certain_choice,
        uncertain_group,
        uncertain_required,
    ]);
    assert_eq!(
        result.uncertain,
        ids(&["future", "uncertain_group", "uncertain_required"])
    );
    assert!(result.excluded_prereq.is_empty());
}

#[test]
fn uncertainty_crosses_swap_aliases_and_cycles_without_looping() {
    let mut root = definition("root");
    root.potential = Some(compiled(leaf("future")));
    root.technology_swaps.push(SwapVariant {
        active_id: "alias".into(),
        trigger: compiled(leaf("true")),
        area: None,
    });
    let mut a = definition("a");
    a.prerequisites.all_of = ids(&["alias", "b"]);
    let mut b = definition("b");
    b.prerequisites.all_of = ids(&["a"]);
    let result = eligible(vec![a, b, root]);
    assert_eq!(result.uncertain, ids(&["a", "b", "root"]));
}

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
