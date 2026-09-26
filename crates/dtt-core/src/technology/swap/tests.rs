use super::*;
use crate::technology::SwapVariant;
use crate::test_support::{Context, catalog, compiled, definition, ids, leaf};

fn variant(active_id: &str, predicate: &str) -> SwapVariant {
    SwapVariant {
        active_id: active_id.into(),
        trigger: compiled(leaf(predicate)),
        area: Some(Area::Society),
    }
}

#[test]
fn uncertain_first_variant_blocks_later_matches_and_preserves_base_area() {
    let mut base = definition("base");
    base.technology_swaps = vec![variant("future", "future"), variant("certain", "true")];
    let catalog = catalog([base]);
    let result = resolve_swaps(
        &catalog,
        &ids(&["base"]),
        &Context,
        SwapUnknownStrategy::KeepBase,
    )
    .unwrap();
    assert_eq!(result.entries[0].outcome, SwapOutcome::Uncertain);
    assert_eq!(result.entries[0].active_id, Id::from("base"));
    assert_eq!(result.entries[0].area, Area::Physics);
    assert!(result.display_id.is_empty());
    assert!(result.display_area.is_empty());
    assert!(matches!(
        resolve_swaps(
            &catalog,
            &ids(&["base"]),
            &Context,
            SwapUnknownStrategy::Error
        ),
        Err(Error::RuleEngine(_))
    ));
}

#[test]
fn false_variants_are_skipped_and_first_certain_match_changes_display_only() {
    let mut base = definition("base");
    base.prerequisites.all_of = ids(&["required"]);
    base.technology_swaps = vec![
        variant("never", "false"),
        variant("first", "true"),
        variant("second", "true"),
    ];
    let catalog = catalog([base]);
    let result = resolve_swaps(
        &catalog,
        &ids(&["base"]),
        &Context,
        SwapUnknownStrategy::KeepBase,
    )
    .unwrap();
    assert_eq!(result.entries[0].outcome, SwapOutcome::Matched);
    assert_eq!(result.display_of(&"base".into()), Id::from("first"));
    assert_eq!(result.display_area[&Id::from("base")], Area::Society);
    assert_eq!(
        catalog.get(&"base".into()).unwrap().prerequisites.all_of,
        ids(&["required"])
    );
}

#[test]
fn unmatched_variants_report_no_match_and_keep_base_display() {
    let mut base = definition("base");
    base.technology_swaps = vec![variant("never", "false")];
    let result = resolve_swaps(
        &catalog([base]),
        &ids(&["base"]),
        &Context,
        SwapUnknownStrategy::KeepBase,
    )
    .unwrap();
    assert_eq!(result.entries[0].outcome, SwapOutcome::NoMatch);
    assert_eq!(result.display_of(&"base".into()), Id::from("base"));
}

#[test]
fn a_swap_cannot_steal_another_eligible_technology_display_id() {
    let mut base = definition("base");
    base.technology_swaps = vec![variant("other", "true")];
    let result = resolve_swaps(
        &catalog([base, definition("other")]),
        &ids(&["base", "other"]),
        &Context,
        SwapUnknownStrategy::KeepBase,
    );
    assert!(matches!(result, Err(Error::RuleEngine(_))));
}
