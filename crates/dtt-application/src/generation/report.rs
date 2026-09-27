use std::collections::{BTreeMap, BTreeSet};

use dtt_core::graph::Graph;
use dtt_core::technology::{
    DeferredTriggerDiagnostic, EligibilityReport, Id, SwapOutcome, SwapResolution,
    UnknownTriggerDiagnostic,
};
use dtt_stellaris::game_data::{GameDataDiagnostic, TechnologyIngest};
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize)]
pub struct GenerationReport {
    pub missing_mod_descriptors: Vec<String>,
    pub eligible: Vec<Id>,
    pub uncertain: Vec<Id>,
    pub excluded_identity: Vec<Id>,
    pub excluded_prereq: Vec<(Id, Vec<Id>)>,
    pub unknown_triggers: Vec<UnknownTriggerDiagnostic>,
    pub deferred_triggers: Vec<DeferredTriggerDiagnostic>,
    pub game_data_diagnostics: Vec<GameDataDiagnostic>,
    pub unhandled_definition_fields: Vec<DefinitionFieldDiagnostic>,
    pub localisation_diagnostics: Vec<LanguageLocalisationDiagnostic>,
    pub cycles_self_refs: Vec<Id>,
    pub cycles_complex: Vec<Vec<Id>>,
    pub swap_matched: usize,
    pub swap_nomatch: usize,
    pub swap_uncertain: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DefinitionFieldDiagnostic {
    pub technology_id: Id,
    pub source: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LanguageLocalisationDiagnostic {
    pub language: String,
    pub source: String,
    pub path: String,
    pub kind: dtt_stellaris::localisation::LocalisationFailureKind,
    pub technical_detail: String,
}

pub(super) fn build_report(
    missing_mod_descriptors: Vec<String>,
    eligibility: EligibilityReport,
    graph: &Graph,
    swaps: &SwapResolution,
    technology_ingest: &TechnologyIngest,
    game_data_diagnostics: Vec<GameDataDiagnostic>,
    localisation_diagnostics: Vec<LanguageLocalisationDiagnostic>,
) -> GenerationReport {
    let cycles = graph.cycle_report().clone();
    let swap_matched = swaps
        .entries
        .iter()
        .filter(|entry| matches!(entry.outcome, SwapOutcome::Matched))
        .count();
    let swap_nomatch = swaps
        .entries
        .iter()
        .filter(|entry| matches!(entry.outcome, SwapOutcome::NoMatch))
        .count();

    let swap_uncertain = swaps
        .entries
        .iter()
        .filter(|entry| matches!(entry.outcome, SwapOutcome::Uncertain))
        .count();
    let mut unhandled_definition_fields: Vec<DefinitionFieldDiagnostic> = technology_ingest
        .metadata
        .iter()
        .filter_map(|(technology_id, metadata)| {
            if metadata.unhandled_fields.is_empty() {
                return None;
            }
            let mut fields = metadata.unhandled_fields.clone();
            fields.sort();
            fields.dedup();
            Some(DefinitionFieldDiagnostic {
                technology_id: technology_id.clone(),
                source: metadata.source.clone(),
                fields,
            })
        })
        .collect();
    unhandled_definition_fields.sort_by(|left, right| left.technology_id.cmp(&right.technology_id));
    let unknown_triggers = merge_unknown_trigger_diagnostics(&eligibility, swaps);
    let deferred_triggers = merge_deferred_trigger_diagnostics(&eligibility, swaps);

    GenerationReport {
        missing_mod_descriptors,
        eligible: eligibility.eligible,
        uncertain: eligibility.uncertain,
        excluded_identity: eligibility.excluded_identity,
        excluded_prereq: eligibility.excluded_prereq,
        unknown_triggers,
        deferred_triggers,
        game_data_diagnostics,
        unhandled_definition_fields,
        localisation_diagnostics,
        cycles_self_refs: cycles.self_references,
        cycles_complex: cycles.complex_cycles,
        swap_matched,
        swap_nomatch,
        swap_uncertain,
    }
}

fn merge_unknown_trigger_diagnostics(
    eligibility: &EligibilityReport,
    swaps: &SwapResolution,
) -> Vec<UnknownTriggerDiagnostic> {
    merge_keyed_diagnostics(
        eligibility.unknown_triggers.iter().map(|diagnostic| {
            (
                diagnostic.reason.clone(),
                diagnostic.occurrences,
                diagnostic.tech_ids.as_slice(),
            )
        }),
        swaps.entries.iter().flat_map(|swap| {
            swap.unknown_triggers.iter().map(|diagnostic| {
                (
                    diagnostic.reason.clone(),
                    diagnostic.occurrences,
                    swap.base_id.clone(),
                )
            })
        }),
    )
    .into_iter()
    .map(|(reason, occurrences, tech_ids)| UnknownTriggerDiagnostic {
        reason,
        occurrences,
        tech_ids,
    })
    .collect()
}

fn merge_deferred_trigger_diagnostics(
    eligibility: &EligibilityReport,
    swaps: &SwapResolution,
) -> Vec<DeferredTriggerDiagnostic> {
    merge_keyed_diagnostics(
        eligibility.deferred_triggers.iter().map(|diagnostic| {
            (
                diagnostic.name.clone(),
                diagnostic.occurrences,
                diagnostic.tech_ids.as_slice(),
            )
        }),
        swaps.entries.iter().flat_map(|swap| {
            swap.deferred_triggers.iter().map(|diagnostic| {
                (
                    diagnostic.name.clone(),
                    diagnostic.occurrences,
                    swap.base_id.clone(),
                )
            })
        }),
    )
    .into_iter()
    .map(|(name, occurrences, tech_ids)| DeferredTriggerDiagnostic {
        name,
        occurrences,
        tech_ids,
    })
    .collect()
}

fn merge_keyed_diagnostics<'a, K: Ord>(
    eligibility: impl IntoIterator<Item = (K, usize, &'a [Id])>,
    swap_items: impl IntoIterator<Item = (K, usize, Id)>,
) -> Vec<(K, usize, Vec<Id>)> {
    let mut merged: BTreeMap<K, (usize, BTreeSet<Id>)> = BTreeMap::new();
    for (key, occurrences, tech_ids) in eligibility {
        let entry = merged.entry(key).or_insert_with(|| (0, BTreeSet::new()));
        entry.0 += occurrences;
        entry.1.extend(tech_ids.iter().cloned());
    }
    for (key, occurrences, tech_id) in swap_items {
        let entry = merged.entry(key).or_insert_with(|| (0, BTreeSet::new()));
        entry.0 += occurrences;
        entry.1.insert(tech_id);
    }
    merged
        .into_iter()
        .map(|(key, (occurrences, tech_ids))| (key, occurrences, tech_ids.into_iter().collect()))
        .collect()
}
