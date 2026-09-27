use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use dtt_application::GameLanguage;

#[derive(Debug, Parser)]
#[command(name = "dtt", version)]
pub struct Cli {
    #[arg(long, global = true, value_name = "LOCALE", value_parser = parse_locale)]
    pub locale: Option<String>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Generate(GenerateArgs),
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
        value_parser = parse_language,
        default_value = "english"
    )]
    pub languages: Vec<GameLanguage>,
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

pub fn command(t: &dtt_i18n::Translator) -> Result<clap::Command, dtt_i18n::TranslationError> {
    use clap::CommandFactory;
    use dtt_i18n::CliMessage as M;
    let template = format!(
        "{{about}}\n\n{}: {{usage}}\n\n{{all-args}}",
        t.cli(M::Usage)?
    );
    let decorate = |mut cmd: clap::Command| -> Result<clap::Command, dtt_i18n::TranslationError> {
        cmd = cmd
            .disable_help_subcommand(true)
            .disable_help_flag(true)
            .disable_version_flag(true)
            .help_template(template.clone())
            .next_help_heading(t.cli(M::Options)?)
            .arg(
                clap::Arg::new("help")
                    .long("help")
                    .short('h')
                    .action(clap::ArgAction::Help)
                    .help(t.cli(M::Help)?),
            )
            .arg(
                clap::Arg::new("version")
                    .long("version")
                    .short('V')
                    .action(clap::ArgAction::Version)
                    .help(t.cli(M::Version)?),
            )
            .version(env!("CARGO_PKG_VERSION"));
        for (id, label) in [
            ("locale", M::Locale),
            ("save_file", M::SaveFile),
            ("stellaris_root", M::GameRoot),
            ("documents_dir", M::Documents),
            ("launcher_db", M::Launcher),
            ("languages", M::Language),
            ("unknown_strategy", M::UnknownStrategy),
            ("swap_unknown_strategy", M::SwapStrategy),
        ] {
            if cmd.get_arguments().any(|arg| arg.get_id() == id) {
                let help = t.cli(label)?;
                let heading = t.cli(if id == "save_file" {
                    M::Arguments
                } else {
                    M::Options
                })?;
                cmd = cmd.mut_arg(id, |arg| {
                    arg.help(help)
                        .help_heading(heading)
                        .hide_default_value(true)
                        .hide_possible_values(true)
                });
            }
        }
        Ok(cmd)
    };
    let mut cmd = Cli::command();
    let children = cmd
        .get_subcommands()
        .cloned()
        .map(|child| {
            let label = if child.get_name() == "generate" {
                M::Generate
            } else {
                M::DetectPaths
            };
            decorate(child.about(t.cli(label)?))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for child in children {
        let name = child.get_name().to_string();
        cmd = cmd.mut_subcommand(name, |_| child);
    }
    let mut cmd = decorate(
        cmd.about(t.cli(M::About)?)
            .subcommand_help_heading(t.cli(M::Commands)?),
    )?;
    let seen = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    cmd = cmd.mut_arg("locale", |arg| {
        arg.value_parser(move |value: &str| {
            if seen.swap(true, std::sync::atomic::Ordering::Relaxed) {
                return Err("--locale".to_string());
            }
            parse_locale(value)
        })
    });
    Ok(cmd)
}

/// Skip option values using clap's command definition, retaining the original OsStrings.
pub fn prefetch_locale(args: &[std::ffi::OsString]) -> Option<std::ffi::OsString> {
    use clap::CommandFactory;
    let mut root = Cli::command();
    root.build();
    let mut command = &root;
    let mut index = 1;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            break;
        }
        if let Some(text) = arg.to_str() {
            if text == "--locale" {
                return args.get(index + 1).cloned();
            }
            if let Some(value) = text.strip_prefix("--locale=") {
                return Some(value.into());
            }
            if let Some(child) = command.find_subcommand(text) {
                command = child;
            } else {
                let option = command.get_arguments().find(|option| {
                    option
                        .get_long()
                        .is_some_and(|long| text == format!("--{long}"))
                        || option
                            .get_short()
                            .is_some_and(|short| text == format!("-{short}"))
                });
                if option
                    .and_then(|arg| arg.get_num_args())
                    .is_some_and(|range| range.takes_values())
                {
                    index += 1;
                }
            }
        }
        index += 1;
    }
    None
}

fn parse_locale(value: &str) -> Result<String, String> {
    if value == "system" || dtt_i18n::AppLocale::matching(value).is_some() {
        Ok(value.to_owned())
    } else {
        Err("system | en | zh-Hans".into())
    }
}
fn parse_language(value: &str) -> Result<GameLanguage, String> {
    match value {
        "english" => Ok(GameLanguage::English),
        "simp_chinese" => Ok(GameLanguage::SimpChinese),
        _ => Err("english | simp_chinese".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn prefetch(args: &[&str]) -> Option<String> {
        prefetch_locale(&args.iter().map(Into::into).collect::<Vec<_>>())
            .map(|v| v.into_string().unwrap())
    }
    #[test]
    fn prefetch_respects_separator_and_other_option_values() {
        assert_eq!(
            prefetch(&["dtt", "generate", "--", "--locale=zh-Hans"]),
            None
        );
        assert_eq!(
            prefetch(&[
                "dtt",
                "generate",
                "--launcher-db",
                "--locale=zh-Hans",
                "--locale=en"
            ]),
            Some("en".into())
        );
        assert_eq!(prefetch(&["dtt", "generate", "--locale"]), None);
        assert_eq!(
            prefetch(&["dtt", "--locale=en", "--locale=zh-Hans"]),
            Some("en".into())
        );
    }
    #[cfg(unix)]
    #[test]
    fn prefetch_preserves_non_utf8_paths() {
        use std::os::unix::ffi::OsStringExt;
        let args = vec![
            "dtt".into(),
            "generate".into(),
            std::ffi::OsString::from_vec(vec![255]),
            "--locale=zh-Hans".into(),
        ];
        assert_eq!(prefetch_locale(&args), Some("zh-Hans".into()));
        assert_eq!(args[2], std::ffi::OsString::from_vec(vec![255]));
    }
}
