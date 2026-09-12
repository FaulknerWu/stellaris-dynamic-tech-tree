use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use dtt_application::SupportedLanguage;

#[derive(Debug, Parser)]
#[command(
    name = "dtt",
    version,
    about = "从 Stellaris 存档生成 Dynamic Technology Tree 本地化文件"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "读取非铁人文本存档并生成科技树文件")]
    Generate(GenerateArgs),
    #[command(about = "打印自动发现到的 Stellaris 路径")]
    DetectPaths,
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[arg(value_name = "SAVE")]
    pub save_file: PathBuf,
    #[arg(long, value_name = "DIR")]
    pub stellaris_root: Option<PathBuf>,
    #[arg(long, value_name = "DIR")]
    pub documents_dir: Option<PathBuf>,
    #[arg(long, value_name = "FILE")]
    pub launcher_db: Option<PathBuf>,
    #[arg(
        short,
        long = "language",
        value_name = "LANG",
        default_value = "english"
    )]
    pub languages: Vec<SupportedLanguage>,
    #[arg(long, value_enum, default_value = "include-flagged")]
    pub unknown_strategy: CliUnknownStrategy,
    #[arg(long, value_enum, default_value = "keep-base")]
    pub swap_unknown_strategy: CliSwapUnknownStrategy,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum CliUnknownStrategy {
    IncludeFlagged,
    ExcludeStrict,
    Error,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum CliSwapUnknownStrategy {
    KeepBase,
    Error,
}
