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
    pub message: String,
}

impl GenerationReport {
    pub(crate) fn format_text(&self) -> String {
        let mut report = String::new();
        report.push_str("=== dtt-save-report ===\n\n");

        report.push_str("[eligibility]\n");
        report.push_str(&format!("  eligible: {}\n", self.eligible.len()));
        report.push_str(&format!("  uncertain: {}\n", self.uncertain.len()));
        report.push_str(&format!(
            "  excluded (identity gate): {}\n",
            self.excluded_identity.len()
        ));
        report.push_str(&format!(
            "  excluded (prereq closure): {}\n\n",
            self.excluded_prereq.len()
        ));

        if !self.excluded_identity.is_empty() {
            report.push_str("excluded by identity:\n");
            for technology_id in &self.excluded_identity {
                report.push_str(&format!("  - {}\n", technology_id.as_str()));
            }
            report.push('\n');
        }

        if !self.uncertain.is_empty() {
            report.push_str("included with uncertain eligibility:\n");
            for technology_id in &self.uncertain {
                report.push_str(&format!("  - {}\n", technology_id.as_str()));
            }
            report.push('\n');
        }

        if !self.excluded_prereq.is_empty() {
            report.push_str("excluded by prerequisite closure:\n");
            for (technology_id, missing_prerequisites) in &self.excluded_prereq {
                let missing_prerequisites: Vec<&str> =
                    missing_prerequisites.iter().map(Id::as_str).collect();
                report.push_str(&format!(
                    "  - {} (missing: {})\n",
                    technology_id.as_str(),
                    missing_prerequisites.join(", ")
                ));
            }
            report.push('\n');
        }

        report.push_str(&format!(
            "[unknown triggers] predicates={}\n",
            self.unknown_triggers.len()
        ));
        for diagnostic in &self.unknown_triggers {
            let examples: Vec<&str> = diagnostic.tech_ids.iter().take(5).map(Id::as_str).collect();
            report.push_str(&format!(
                "  - {} (occurrences: {}; affected techs: {}; examples: {})\n",
                diagnostic.reason,
                diagnostic.occurrences,
                diagnostic.tech_ids.len(),
                examples.join(", ")
            ));
        }
        report.push('\n');

        report.push_str(&format!(
            "[过程性条件（保留未来可能性）] predicates={}\n",
            self.deferred_triggers.len()
        ));
        for diagnostic in &self.deferred_triggers {
            let examples: Vec<&str> = diagnostic.tech_ids.iter().take(5).map(Id::as_str).collect();
            report.push_str(&format!(
                "  - {} (occurrences: {}; affected techs: {}; examples: {})\n",
                diagnostic.name,
                diagnostic.occurrences,
                diagnostic.tech_ids.len(),
                examples.join(", ")
            ));
        }
        report.push('\n');

        report.push_str(&format!(
            "[game data diagnostics] count={}\n",
            self.game_data_diagnostics.len()
        ));
        for diagnostic in &self.game_data_diagnostics {
            let subject = diagnostic.subject.as_deref().unwrap_or("-");
            let offset = diagnostic
                .byte_offset
                .map(|offset| offset.to_string())
                .unwrap_or_else(|| "-".to_string());
            report.push_str(&format!(
                "  - {} {} {} {} {}: {}\n",
                diagnostic.source,
                diagnostic.category.as_str(),
                subject,
                offset,
                diagnostic.kind.as_str(),
                diagnostic.message
            ));
        }
        report.push('\n');

        report.push_str(&format!(
            "[definition diagnostics] unhandled_definitions={}\n",
            self.unhandled_definition_fields.len()
        ));
        for diagnostic in &self.unhandled_definition_fields {
            report.push_str(&format!(
                "  - {}:{} (unhandled fields: {})\n",
                diagnostic.source,
                diagnostic.technology_id.as_str(),
                diagnostic.fields.join(", ")
            ));
        }
        report.push('\n');

        report.push_str(&format!(
            "[localisation diagnostics] count={}\n",
            self.localisation_diagnostics.len()
        ));
        for diagnostic in &self.localisation_diagnostics {
            report.push_str(&format!(
                "  - {}:{}:{}: {}\n",
                diagnostic.language, diagnostic.source, diagnostic.path, diagnostic.message
            ));
        }
        report.push('\n');

        report.push_str(&format!(
            "[cycles] self_references={} complex={}\n",
            self.cycles_self_refs.len(),
            self.cycles_complex.len()
        ));
        for technology_id in &self.cycles_self_refs {
            report.push_str(&format!("  - self: {}\n", technology_id.as_str()));
        }
        for cycle in &self.cycles_complex {
            let technology_ids: Vec<&str> = cycle.iter().map(Id::as_str).collect();
            report.push_str(&format!("  - cycle: {}\n", technology_ids.join(" -> ")));
        }
        report.push('\n');

        report.push_str(&format!(
            "[swap] matched={} no_match={} uncertain={}\n\n",
            self.swap_matched, self.swap_nomatch, self.swap_uncertain,
        ));
        report
    }
}

pub(super) fn build_report(
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
