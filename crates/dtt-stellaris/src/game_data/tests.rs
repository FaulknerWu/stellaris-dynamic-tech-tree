use std::fs;

use dtt_core::condition::UnknownStrategy;
use dtt_core::empire::Snapshot;
use dtt_core::technology::{SwapOutcome, SwapUnknownStrategy, evaluate_eligibility, resolve_swaps};

use super::load_game_data;
use crate::analysis::World;
use crate::load_order::{Manifest, SourceEntry};

struct Fixture {
    _directory: tempfile::TempDir,
    manifest: Manifest,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let sources = ["base", "first_mod", "last_mod"]
            .into_iter()
            .enumerate()
            .map(|(index, name)| SourceEntry {
                index: index as u32,
                name: name.into(),
                root: directory.path().join(name),
                replace_paths: Vec::new(),
            })
            .collect();
        Self {
            _directory: directory,
            manifest: Manifest {
                sources,
                missing_mod_descriptors: Vec::new(),
            },
        }
    }

    fn write(&self, source: usize, relative: &str, content: &str) {
        let path = self.manifest.sources[source].root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
}

#[test]
fn later_file_replaces_all_earlier_definitions_at_the_same_path() {
    let fixture = Fixture::new();
    fixture.write(
        0,
        "common/technology/shared.txt",
        "removed = { area = physics tier = 1 } retained = { area = physics tier = 1 }",
    );
    fixture.write(
        1,
        "common/technology/shared.txt",
        "also_removed = { area = physics tier = 2 } retained = { area = physics tier = 2 }",
    );
    fixture.write(
        2,
        "common/technology/shared.txt",
        "retained = { area = physics tier = 3 }",
    );
    fixture.write(
        0,
        "common/technology/independent.txt",
        "independent = { area = society tier = 1 }",
    );
    let data = load_game_data(&fixture.manifest).unwrap();
    let catalog = &data.value.technologies.catalog;
    assert_eq!(catalog.len(), 2);
    assert!(catalog.contains(&"independent".into()));
    assert_eq!(catalog.get(&"retained".into()).unwrap().tier, 3);
}

#[test]
fn empty_override_removes_a_file_without_hiding_other_files() {
    let fixture = Fixture::new();
    fixture.write(
        0,
        "common/technology/removed.txt",
        "removed = { area = physics tier = 1 }",
    );
    fixture.write(1, "common/technology/removed.txt", "");
    fixture.write(
        0,
        "common/technology/retained.txt",
        "retained = { area = physics tier = 1 }",
    );
    let data = load_game_data(&fixture.manifest).unwrap();
    assert_eq!(data.value.technologies.catalog.len(), 1);
    assert!(data.value.technologies.catalog.contains(&"retained".into()));
}

#[test]
fn recursive_localisation_loading_also_discards_replaced_files() {
    let fixture = Fixture::new();
    fixture.write(
        0,
        "common/technology/test.txt",
        "a = { area = physics tier = 1 } b = { area = physics tier = 1 }",
    );
    fixture.write(
        0,
        "localisation/english/shared_l_english.yml",
        "l_english:\n a_desc:0 \"old A\"\n b_desc:0 \"old B\"",
    );
    fixture.write(
        1,
        "localisation/english/shared_l_english.yml",
        "l_english:\n b_desc:0 \"new B\"",
    );
    let data = load_game_data(&fixture.manifest).unwrap();
    let localisation = crate::localisation::ingest(
        &fixture.manifest,
        "english",
        &data.value.technologies.catalog,
    )
    .unwrap();
    assert!(!localisation.descriptions.contains_key(&"a".into()));
    assert_eq!(localisation.descriptions[&"b".into()], "new B");
}

#[test]
fn replace_path_and_distinct_file_definition_precedence_are_preserved() {
    let mut fixture = Fixture::new();
    fixture.write(
        0,
        "common/technology/base.txt",
        "removed = { area = physics tier = 1 }",
    );
    fixture.manifest.sources[1]
        .replace_paths
        .push("common/technology".into());
    fixture.write(
        1,
        "common/technology/z.txt",
        "retained = { area = physics tier = 2 }",
    );
    fixture.write(
        2,
        "common/technology/a.txt",
        "retained = { area = physics tier = 3 }",
    );
    let data = load_game_data(&fixture.manifest).unwrap();
    assert_eq!(data.value.technologies.catalog.len(), 1);
    assert_eq!(
        data.value
            .technologies
            .catalog
            .get(&"retained".into())
            .unwrap()
            .tier,
        3
    );
}

#[test]
fn failed_inline_conditions_obey_all_unknown_strategies() {
    for condition in [
        "inline_script = missing",
        "inline_script = loop_a",
        "inline_script = {}",
        "NOT = { inline_script = missing }",
        "OR = { always = no inline_script = missing }",
        "broken_trigger = yes",
    ] {
        let fixture = Fixture::new();
        fixture.write(
            0,
            "common/inline_scripts/loop_a.txt",
            "inline_script = loop_b",
        );
        fixture.write(
            0,
            "common/inline_scripts/loop_b.txt",
            "inline_script = loop_a",
        );
        fixture.write(
            0,
            "common/scripted_triggers/test.txt",
            "broken_trigger = { inline_script = missing }",
        );
        fixture.write(
            0,
            "common/technology/test.txt",
            &format!("probe = {{ area = physics tier = 1 potential = {{ {condition} }} }}"),
        );
        let data = load_game_data(&fixture.manifest).unwrap();
        assert!(!data.diagnostics.is_empty());
        let snapshot = Snapshot::default();
        let world = World {
            snapshot: &snapshot,
            ship_kinds: &data.value.ship_kinds,
        };
        let context = world.context();
        let catalog = &data.value.technologies.catalog;
        let included =
            evaluate_eligibility(catalog, &context, UnknownStrategy::IncludeFlagged).unwrap();
        assert_eq!(included.eligible, vec!["probe".into()], "{condition}");
        assert_eq!(included.uncertain, included.eligible, "{condition}");
        assert!(!included.unknown_triggers.is_empty(), "{condition}");
        let strict =
            evaluate_eligibility(catalog, &context, UnknownStrategy::ExcludeStrict).unwrap();
        assert!(strict.eligible.is_empty(), "{condition}");
        assert!(
            evaluate_eligibility(catalog, &context, UnknownStrategy::Error).is_err(),
            "{condition}"
        );
    }
}

#[test]
fn failed_inline_swap_trigger_does_not_select_a_variant() {
    let fixture = Fixture::new();
    fixture.write(0, "common/technology/test.txt", "probe = { area = physics tier = 1 technology_swap = { name = variant trigger = { inline_script = missing } } }");
    let data = load_game_data(&fixture.manifest).unwrap();
    let snapshot = Snapshot::default();
    let world = World {
        snapshot: &snapshot,
        ship_kinds: &data.value.ship_kinds,
    };
    let context = world.context();
    let catalog = &data.value.technologies.catalog;
    let eligible = ["probe".into()];
    let swaps = resolve_swaps(catalog, &eligible, &context, SwapUnknownStrategy::KeepBase).unwrap();
    assert_eq!(swaps.display_of(&eligible[0]), eligible[0]);
    assert!(matches!(swaps.entries[0].outcome, SwapOutcome::Uncertain));
    assert!(resolve_swaps(catalog, &eligible, &context, SwapUnknownStrategy::Error).is_err());
}

#[test]
fn incomplete_top_level_inline_definitions_are_skipped() {
    let fixture = Fixture::new();
    fixture.write(
        0,
        "common/technology/test.txt",
        "probe = { area = physics tier = 1 inline_script = missing }",
    );
    let data = load_game_data(&fixture.manifest).unwrap();
    assert!(data.value.technologies.catalog.is_empty());
    assert!(!data.diagnostics.is_empty());
}

#[test]
fn incomplete_inline_swap_definitions_are_skipped() {
    let fixture = Fixture::new();
    fixture.write(0, "common/technology/test.txt", "probe = { area = physics tier = 1 technology_swap = { name = variant inline_script = missing } }");
    let data = load_game_data(&fixture.manifest).unwrap();
    let definition = data
        .value
        .technologies
        .catalog
        .get(&"probe".into())
        .unwrap();
    assert!(definition.technology_swaps.is_empty());
    assert!(!data.diagnostics.is_empty());
}

#[test]
fn successfully_expanded_inline_conditions_keep_their_meaning() {
    let fixture = Fixture::new();
    fixture.write(0, "common/inline_scripts/deny.txt", "always = no");
    fixture.write(0, "common/inline_scripts/empty.txt", "");
    fixture.write(0, "common/technology/test.txt", "denied = { area = physics tier = 1 potential = { inline_script = deny } } allowed = { area = physics tier = 1 potential = { inline_script = empty } }");
    let data = load_game_data(&fixture.manifest).unwrap();
    let snapshot = Snapshot::default();
    let world = World {
        snapshot: &snapshot,
        ship_kinds: &data.value.ship_kinds,
    };
    let report = evaluate_eligibility(
        &data.value.technologies.catalog,
        &world.context(),
        UnknownStrategy::Error,
    )
    .unwrap();
    assert_eq!(report.eligible, vec!["allowed".into()]);
}
