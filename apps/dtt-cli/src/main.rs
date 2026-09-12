#![forbid(unsafe_code)]

mod args;
mod commands;

use anyhow::Result;
use args::{Cli, Commands};
use clap::Parser;

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Generate(args) => commands::generate::run(args),
        Commands::DetectPaths => commands::detect_paths::run(),
    }
}
