use anyhow::{Context, Result};
use dtt_application::{
    GenerationStatus, ResolveGenerationSettingsRequest, RunGenerationRequest, SwapUnknownStrategy,
    UnknownStrategy, resolve_generation_settings, run_generation,
};

use crate::args::{CliSwapUnknownStrategy, CliUnknownStrategy, GenerateArgs};

pub fn run(args: GenerateArgs) -> Result<()> {
    let settings = resolve_generation_settings(&ResolveGenerationSettingsRequest {
        stellaris_root: args.stellaris_root.clone(),
        documents_dir: args.documents_dir.clone(),
        launcher_db: args.launcher_db.clone(),
        languages: args.languages.clone(),
        unknown_strategy: args.unknown_strategy.into(),
        swap_unknown_strategy: args.swap_unknown_strategy.into(),
    })
    .context("invalid configuration")?;
    println!("使用群星本体目录: {}", settings.stellaris_root);
    println!("使用启动器数据库: {}", settings.launcher_db);
    println!("输出位置: 当前可执行文件所在目录的 localisation 子目录");

    let request = RunGenerationRequest::from_save(args.save_file, settings);
    let outcome = run_generation(&request).context("failed to generate technology tree")?;
    println!("生成状态: {}", status_label(outcome.output.status));
    println!(
        "科技定义: {}，可用科技: {}，数据源: {}",
        outcome.technology_count,
        outcome.report.eligible.len(),
        outcome.source_count
    );
    println!(
        "条件替换: 已匹配={}，未匹配={}，不确定并保留基础显示={}",
        outcome.report.swap_matched, outcome.report.swap_nomatch, outcome.report.swap_uncertain
    );
    let parse_failures = outcome.report.game_data_diagnostics.len();
    if parse_failures > 0 {
        println!("Game data diagnostics: {parse_failures}");
    }
    if let Some(path) = &outcome.output.report_path {
        println!("Report: {path}");
    }
    if !outcome.output.written.is_empty() {
        println!("已写出文件:");
        for path in &outcome.output.written {
            println!("  {path}");
        }
    }
    if !outcome.output.removed.is_empty() {
        println!("已清理过期文件:");
        for path in &outcome.output.removed {
            println!("  {path}");
        }
    }
    if !outcome.output.failed.is_empty() {
        println!("写出失败:");
        for failure in &outcome.output.failed {
            println!("  {failure}");
        }
    }
    Ok(())
}

fn status_label(status: GenerationStatus) -> &'static str {
    match status {
        GenerationStatus::Success => "success",
        GenerationStatus::Incomplete => "incomplete",
    }
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
