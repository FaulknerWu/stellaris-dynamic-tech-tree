use crate::error::SettingsIssue;
use std::collections::HashMap;
use std::path::Path;

use dtt_core::graph::Graph;
use dtt_core::technology::{Catalog, Id, SwapResolution, evaluate_eligibility, resolve_swaps};
use dtt_stellaris::analysis::World;
use dtt_stellaris::game_data::load_game_data;
use dtt_stellaris::load_order::{Manifest, resolve};
use dtt_stellaris::localisation;
use dtt_stellaris::output::{GameLanguage, WriteRequest, render_tree_content, write};

use super::report::{LanguageLocalisationDiagnostic, build_report};
use super::{GenerationSource, RunGenerationRequest, RunGenerationResult, RunOutcome};
use crate::error::{Error, Result};
use crate::progress::{GenerationStage, GenerationStatus};
use crate::save::open_text_save;

pub fn run_generation(request: &RunGenerationRequest) -> Result<RunGenerationResult> {
    request.settings.validate()?;
    validate_paths(request)?;

    notify_stage(request, GenerationStage::SaveParse)?;
    let snapshot = match &request.source {
        GenerationSource::Save { path, country_id } => {
            let gamestate = open_text_save(path)?;
            match country_id {
                Some(country_id) => {
                    dtt_stellaris::save::extract_snapshot_for_country(&gamestate, *country_id)?
                }
                None => {
                    let candidates = dtt_stellaris::save::player_country_candidates(&gamestate)?;
                    match candidates.len() {
                        0 => return Err(Error::PlayerCountryMissing),
                        1 => dtt_stellaris::save::extract_snapshot_for_country(
                            &gamestate,
                            candidates[0].country_id,
                        )?,
                        _ => return Err(Error::PlayerCountryRequired),
                    }
                }
            }
        }
        GenerationSource::Snapshot(snapshot) => snapshot.as_ref().clone(),
    };

    notify_stage(request, GenerationStage::LoadOrder)?;
    let manifest = resolve(
        Path::new(&request.settings.stellaris_root),
        Path::new(&request.settings.launcher_db),
    )?;
    notify_stage(request, GenerationStage::IngestTech)?;
    let game_data = load_game_data(&manifest)?;
    if game_data.value.technologies.catalog.is_empty() {
        return Err(Error::NoTechnologyDefinitions);
    }

    notify_stage(request, GenerationStage::Relations)?;
    let world = World {
        snapshot: &snapshot,
        ship_kinds: &game_data.value.ship_kinds,
    };
    let context = world.context();
    let technologies = &game_data.value.technologies.catalog;

    let eligibility =
        evaluate_eligibility(technologies, &context, request.settings.unknown_strategy)?;
    let swaps = resolve_swaps(
        technologies,
        &eligibility.eligible,
        &context,
        request.settings.swap_unknown_strategy,
    )?;
    let graph = Graph::build(technologies, &eligibility.eligible);

    notify_stage(request, GenerationStage::IngestL10n)?;
    let description_ingest = effective_descriptions_by_language(
        &manifest,
        &request.settings.languages,
        technologies,
        &swaps,
        &eligibility.eligible,
    )?;
    notify_stage(request, GenerationStage::Render)?;
    let render_results_by_language = render_all(
        technologies,
        &graph,
        &swaps,
        &eligibility.eligible,
        &request.settings.languages,
        request,
    )?;

    notify_stage(request, GenerationStage::Cycles)?;
    let report = build_report(
        manifest.missing_mod_descriptors.clone(),
        eligibility,
        &graph,
        &swaps,
        &game_data.value.technologies,
        game_data.diagnostics,
        description_ingest.diagnostics,
    );

    notify_stage(request, GenerationStage::WriteOutput)?;
    let output_root_dir = crate::environment::output_directory()?;
    let tiers: HashMap<Id, i32> = report
        .eligible
        .iter()
        .filter_map(|technology_id| {
            technologies
                .get(technology_id)
                .map(|technology| (technology_id.clone(), technology.tier))
        })
        .collect();
    let report_body = super::render_report(&report, request.presentation.report_locale)?;
    let written = write(&WriteRequest {
        eligible: &report.eligible,
        render_results_by_language: &render_results_by_language,
        original_descriptions_by_language: &description_ingest.by_language,
        tiers: &tiers,
        display_ids: &swaps.display_id,
        languages: &request.settings.languages,
        output_root_dir: &output_root_dir,
        report_body: &report_body,
    });
    let output = RunOutcome {
        status: if written.complete {
            GenerationStatus::Success
        } else {
            GenerationStatus::Incomplete
        },
        written: written.written,
        removed: written.removed,
        failed: written.failed,
        report_path: written.report_path,
    };
    notify_stage(request, GenerationStage::Done)?;

    Ok(RunGenerationResult {
        source_count: manifest.len(),
        technology_count: technologies.len(),
        output,
        report,
    })
}

fn notify_stage(request: &RunGenerationRequest, stage: GenerationStage) -> Result<()> {
    if request.cancellation.is_cancelled() {
        return Err(Error::Cancelled);
    }
    if let Some(progress) = request.progress.as_ref() {
        progress(stage);
    }
    Ok(())
}

fn validate_paths(request: &RunGenerationRequest) -> Result<()> {
    if let GenerationSource::Save { path, .. } = &request.source
        && !path.is_file()
    {
        return Err(Error::SaveUnavailable(path.clone()));
    }
    let stellaris_root = Path::new(&request.settings.stellaris_root);
    if !stellaris_root.is_dir() {
        return Err(Error::Settings(SettingsIssue::InvalidPath {
            field: "stellaris_root",
            path: stellaris_root.to_path_buf(),
        }));
    }
    let launcher_db = Path::new(&request.settings.launcher_db);
    if !launcher_db.is_file() {
        return Err(Error::Settings(SettingsIssue::InvalidPath {
            field: "launcher_db",
            path: launcher_db.to_path_buf(),
        }));
    }
    Ok(())
}

fn render_all(
    technologies: &Catalog,
    graph: &Graph,
    swaps: &SwapResolution,
    eligible: &[Id],
    languages: &[GameLanguage],
    request: &RunGenerationRequest,
) -> Result<HashMap<GameLanguage, HashMap<Id, String>>> {
    let mut by_language = HashMap::new();
    for lang in languages {
        let translator = dtt_i18n::Translator::new(lang.locale(), dtt_i18n::Domain::Game)?;
        let mut trees = HashMap::new();
        for id in eligible {
            trees.insert(
                id.clone(),
                render_tree_content(
                    id,
                    graph,
                    technologies,
                    swaps,
                    &request.render_limits,
                    &translator,
                )?,
            );
        }
        by_language.insert(*lang, trees);
    }
    Ok(by_language)
}

struct DescriptionIngest {
    by_language: HashMap<GameLanguage, HashMap<Id, String>>,
    diagnostics: Vec<LanguageLocalisationDiagnostic>,
}

fn effective_descriptions_by_language(
    manifest: &Manifest,
    languages: &[GameLanguage],
    technologies: &Catalog,
    swaps: &SwapResolution,
    eligible: &[Id],
) -> Result<DescriptionIngest> {
    let mut by_language = HashMap::new();
    let mut diagnostics = Vec::new();
    for language in languages {
        let localisation = localisation::ingest(manifest, language.code(), technologies)?;
        diagnostics.extend(localisation.diagnostics.iter().cloned().map(|diagnostic| {
            LanguageLocalisationDiagnostic {
                language: language.code().to_string(),
                source: diagnostic.source,
                path: diagnostic.path,
                kind: diagnostic.kind,
                technical_detail: diagnostic.technical_detail,
            }
        }));
        let mut descriptions = HashMap::new();
        for id in eligible {
            let display_id = swaps.display_of(id);

            let description = localisation
                .descriptions
                .get(&display_id)
                .or_else(|| localisation.descriptions.get(id));
            if let Some(description) = description {
                descriptions.insert(id.clone(), description.clone());
            }
        }
        by_language.insert(*language, descriptions);
    }
    Ok(DescriptionIngest {
        by_language,
        diagnostics,
    })
}
