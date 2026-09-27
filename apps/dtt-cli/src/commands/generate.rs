use anyhow::Result;
use dtt_application::{
    GenerationStatus, ResolveGenerationSettingsRequest, RunGenerationRequest, SwapUnknownStrategy,
    UnknownStrategy, resolve_generation_settings, run_generation,
};

use crate::args::{CliSwapUnknownStrategy, CliUnknownStrategy, GenerateArgs};

pub fn run(
    args: GenerateArgs,
    locale: dtt_i18n::AppLocale,
    t: &dtt_i18n::Translator,
) -> Result<()> {
    use dtt_i18n::CliMessage as M;
    let settings = resolve_generation_settings(&ResolveGenerationSettingsRequest {
        stellaris_root: args.stellaris_root,
        documents_dir: args.documents_dir,
        launcher_db: args.launcher_db,
        languages: args.languages,
        unknown_strategy: args.unknown_strategy.into(),
        swap_unknown_strategy: args.swap_unknown_strategy.into(),
    })?;
    println!("{}: {}", t.cli(M::GameRoot)?, settings.stellaris_root);
    println!("{}: {}", t.cli(M::Launcher)?, settings.launcher_db);
    println!("{}", t.cli(M::Output)?);
    let mut request = RunGenerationRequest::from_save(args.save_file, settings);
    request.presentation.report_locale = locale;
    let stages = [
        dtt_application::GenerationStage::SaveParse,
        dtt_application::GenerationStage::LoadOrder,
        dtt_application::GenerationStage::IngestTech,
        dtt_application::GenerationStage::Relations,
        dtt_application::GenerationStage::IngestL10n,
        dtt_application::GenerationStage::Render,
        dtt_application::GenerationStage::Cycles,
        dtt_application::GenerationStage::WriteOutput,
        dtt_application::GenerationStage::Done,
    ];
    let messages = stages
        .into_iter()
        .map(|stage| {
            Ok((
                stage,
                t.stage(match stage {
                    dtt_application::GenerationStage::SaveParse => {
                        dtt_i18n::StageMessage::SaveParse
                    }
                    dtt_application::GenerationStage::LoadOrder => {
                        dtt_i18n::StageMessage::LoadOrder
                    }
                    dtt_application::GenerationStage::IngestTech => {
                        dtt_i18n::StageMessage::IngestTech
                    }
                    dtt_application::GenerationStage::Relations => {
                        dtt_i18n::StageMessage::Relations
                    }
                    dtt_application::GenerationStage::IngestL10n => {
                        dtt_i18n::StageMessage::IngestL10n
                    }
                    dtt_application::GenerationStage::Render => dtt_i18n::StageMessage::Render,
                    dtt_application::GenerationStage::Cycles => dtt_i18n::StageMessage::Cycles,
                    dtt_application::GenerationStage::WriteOutput => {
                        dtt_i18n::StageMessage::WriteOutput
                    }
                    dtt_application::GenerationStage::Done => dtt_i18n::StageMessage::Done,
                })?,
            ))
        })
        .collect::<Result<Vec<_>, dtt_i18n::TranslationError>>()?;
    request.progress = Some(std::sync::Arc::new(move |stage| {
        if let Some((_, message)) = messages.iter().find(|(candidate, _)| *candidate == stage) {
            eprintln!("{message}");
        }
    }));
    let result = run_generation(&request)?;
    println!(
        "{}",
        t.cli(match result.output.status {
            GenerationStatus::Success => M::Success,
            GenerationStatus::Incomplete => M::Incomplete,
        })?
    );
    println!(
        "{}",
        t.cli_summary(
            result.technology_count,
            result.report.eligible.len(),
            result.source_count
        )?
    );
    print!(
        "{}",
        dtt_application::render_report(&result.report, locale)?
    );
    for (label, paths) in [
        (M::Written, &result.output.written),
        (M::Removed, &result.output.removed),
    ] {
        if !paths.is_empty() {
            println!("{}:", t.cli(label)?);
        }
        for path in paths {
            println!("  {path}");
        }
    }
    if !result.output.failed.is_empty() {
        println!("{}:", t.cli(M::Failed)?);
    }
    for failure in result.output.failed {
        println!("  {}: {}", failure.path, failure.technical_detail);
    }
    Ok(())
}

impl From<CliUnknownStrategy> for UnknownStrategy {
    fn from(value: CliUnknownStrategy) -> Self {
        match value {
            CliUnknownStrategy::IncludeFlagged => Self::IncludeFlagged,
            CliUnknownStrategy::ExcludeStrict => Self::ExcludeStrict,
            CliUnknownStrategy::Error => Self::Error,
        }
    }
}

impl From<CliSwapUnknownStrategy> for SwapUnknownStrategy {
    fn from(value: CliSwapUnknownStrategy) -> Self {
        match value {
            CliSwapUnknownStrategy::KeepBase => Self::KeepBase,
            CliSwapUnknownStrategy::Error => Self::Error,
        }
    }
}
