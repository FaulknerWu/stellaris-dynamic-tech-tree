use super::*;
use dtt_core::condition::{ComparisonOperator, Condition, PredicateArgument, TruthBounds};

#[test]
fn builtin_parameter_comparisons_survive_parsing_and_script_binding() {
    let fixture = Fixture::new();
    fixture.write(
        0,
        "common/scripted_triggers/test.txt",
        r#"
        expenses = { resource_expenses_compare = { resource = $RESOURCE$ value > $LIMIT$ } }
    "#,
    );
    fixture.write(0, "common/technology/test.txt", r#"
        @threshold = 0
        equal = { area = society tier = 1 potential = { resource_expenses_compare = { resource = consumer_goods value = @threshold } } }
        greater = { area = society tier = 1 potential = { resource_expenses_compare = { resource = consumer_goods value > @threshold } } }
        bound = { area = society tier = 1 potential = { expenses = { RESOURCE = consumer_goods LIMIT = @threshold } } }
    "#);
    let data = load_game_data(&fixture.manifest).unwrap();
    assert!(data.diagnostics.is_empty());
    let catalog = &data.value.technologies.catalog;
    let condition = |id: &str| {
        &catalog
            .get(&id.into())
            .unwrap()
            .potential
            .as_ref()
            .unwrap()
            .condition
    };
    assert_ne!(condition("equal"), condition("greater"));
    let Condition::Predicate(predicate) = condition("greater") else {
        panic!("expected predicate")
    };
    let PredicateArgument::Parameters(fields) = &predicate.argument else {
        panic!("expected fields")
    };
    assert_eq!(fields[1].name, "value");
    assert_eq!(fields[1].operator, ComparisonOperator::GreaterThan);
    assert_eq!(fields[1].value, "0");
    let Condition::Scripted { inner, .. } = condition("bound") else {
        panic!("expected scripted condition")
    };
    assert_eq!(inner.as_ref(), condition("greater"));
}

#[test]
fn documented_dna_and_expense_blocks_are_deferred_in_every_strategy() {
    for body in [
        "has_dna = { ship_category = tiyanki }",
        "has_dna = { ship_category = tiyanki rarity = rare }",
        "NOT = { has_dna = { ship_category = tiyanki } }",
        "resource_expenses_compare = { resource = consumer_goods value > 0 }",
        "resource_expenses_compare = { resource = food category = pops value = 0 }",
        "resource_expenses_compare = { resource = food value >= 0.5 }",
        "resource_expenses_compare = { resource = food value <= 2 }",
        "resource_expenses_compare = { resource = food value < 2 }",
        "resource_expenses_compare = { resource = food value != 0 }",
    ] {
        let fixture = Fixture::new();
        fixture.write(
            0,
            "common/technology/test.txt",
            &format!("probe = {{ area = society tier = 1 potential = {{ {body} }} }}"),
        );
        let data = load_game_data(&fixture.manifest).unwrap();
        assert!(data.diagnostics.is_empty(), "{body}");
        let snapshot = Snapshot::default();
        let world = World {
            snapshot: &snapshot,
            ship_kinds: &data.value.ship_kinds,
        };
        let catalog = &data.value.technologies.catalog;
        let context = world.context();
        let evaluation = catalog
            .get(&"probe".into())
            .unwrap()
            .potential
            .as_ref()
            .unwrap()
            .evaluate_detailed(&context);
        assert_eq!(evaluation.bounds, TruthBounds::deferred(), "{body}");
        for strategy in [
            UnknownStrategy::IncludeFlagged,
            UnknownStrategy::ExcludeStrict,
            UnknownStrategy::Error,
        ] {
            let report = evaluate_eligibility(catalog, &context, strategy).unwrap();
            assert_eq!(report.eligible, vec!["probe".into()], "{body}");
            assert_eq!(report.uncertain, report.eligible, "{body}");
            assert!(report.unknown_triggers.is_empty(), "{body}");
            assert!(!report.deferred_triggers.is_empty(), "{body}");
        }
    }
}

#[test]
fn malformed_or_unverified_builtin_blocks_stay_unknown() {
    for body in [
        "has_dna = 10",
        "has_dna = {}",
        "has_dna = { rarity = rare }",
        "has_dna = { ship_category > tiyanki }",
        "has_dna = { ship_category = tiyanki extra = yes }",
        "has_dna = { ship_category = @missing }",
        "has_dna = { ship_category = tiyanki ship_category = voidworm }",
        "has_dna > { ship_category = tiyanki }",
        "resource_expenses_compare = 0",
        "resource_expenses_compare = { resource = food }",
        "resource_expenses_compare = { value > 0 }",
        "resource_expenses_compare = { resource > food value > 0 }",
        "resource_expenses_compare = { resource = food value > NaN }",
        "resource_expenses_compare = { resource = food value > inf }",
        "resource_expenses_compare = { resource = food value > @missing }",
        "resource_expenses_compare = { resource = food value ?= 0 }",
        "resource_expenses_compare = { resource = food value > 0 extra = yes }",
        "resource_expenses_compare = { resource = food value = { amount = 0 } }",
        "species = { has_dna = { ship_category = tiyanki } }",
    ] {
        let fixture = Fixture::new();
        fixture.write(
            0,
            "common/technology/test.txt",
            &format!("probe = {{ area = society tier = 1 potential = {{ {body} }} }}"),
        );
        let data = load_game_data(&fixture.manifest).unwrap();
        let snapshot = Snapshot::default();
        let world = World {
            snapshot: &snapshot,
            ship_kinds: &data.value.ship_kinds,
        };
        let catalog = &data.value.technologies.catalog;
        let context = world.context();
        let report =
            evaluate_eligibility(catalog, &context, UnknownStrategy::IncludeFlagged).unwrap();
        assert_eq!(report.uncertain, vec!["probe".into()], "{body}");
        assert!(!report.unknown_triggers.is_empty(), "{body}");
        assert!(
            evaluate_eligibility(catalog, &context, UnknownStrategy::ExcludeStrict)
                .unwrap()
                .eligible
                .is_empty(),
            "{body}"
        );
        assert!(
            evaluate_eligibility(catalog, &context, UnknownStrategy::Error).is_err(),
            "{body}"
        );
    }
}

#[test]
fn scripted_substitution_arguments_must_be_assignments() {
    for (operator, valid) in [("=", true), (">", false), ("!=", false)] {
        let fixture = Fixture::new();
        fixture.write(
            0,
            "common/scripted_triggers/test.txt",
            "guard = { always = $VALUE$ }",
        );
        fixture.write(0, "common/technology/test.txt", &format!("probe = {{ area = society tier = 1 potential = {{ guard = {{ VALUE {operator} yes }} }} }}"));
        let data = load_game_data(&fixture.manifest).unwrap();
        let snapshot = Snapshot::default();
        let world = World {
            snapshot: &snapshot,
            ship_kinds: &data.value.ship_kinds,
        };
        let report = evaluate_eligibility(
            &data.value.technologies.catalog,
            &world.context(),
            UnknownStrategy::IncludeFlagged,
        )
        .unwrap();
        assert_eq!(report.unknown_triggers.is_empty(), valid);
        assert_eq!(report.uncertain.is_empty(), valid);
    }
}

#[test]
fn wilderness_science_lab_display_is_stable_across_weather_control_branches() {
    let fixture = Fixture::new();
    fixture.write(
        0,
        "common/scripted_triggers/test.txt",
        "is_wilderness_empire = { has_origin = origin_wilderness }",
    );
    fixture.write(0, "common/technology/test.txt", r#"
        tech_basic_science_lab_3 = {
            area = physics tier = 3
            technology_swap = {
                name = tech_wilderness_science_lab_3
                trigger = { is_wilderness_empire = yes NOT = { has_ascension_perk = ap_weather_control } }
            }
            technology_swap = {
                name = tech_basic_science_lab_3
                trigger = { is_wilderness_empire = no has_ascension_perk = ap_weather_control }
            }
            technology_swap = {
                name = tech_wilderness_science_lab_3
                trigger = { is_wilderness_empire = yes has_ascension_perk = ap_weather_control }
            }
        }
    "#);
    let data = load_game_data(&fixture.manifest).unwrap();
    let mut snapshot = Snapshot::default();
    snapshot.government.origin = Some("origin_wilderness".into());
    for perks in [vec![], vec!["ap_weather_control".into()]] {
        snapshot.ascension_perks = perks;
        let world = World {
            snapshot: &snapshot,
            ship_kinds: &data.value.ship_kinds,
        };
        let id = "tech_basic_science_lab_3".into();
        let swaps = resolve_swaps(
            &data.value.technologies.catalog,
            std::slice::from_ref(&id),
            &world.context(),
            SwapUnknownStrategy::Error,
        )
        .unwrap();
        assert_eq!(
            swaps.display_of(&id),
            "tech_wilderness_science_lab_3".into()
        );
        assert_eq!(swaps.entries[0].outcome, SwapOutcome::Matched);
    }
}
