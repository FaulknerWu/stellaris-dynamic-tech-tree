#![forbid(unsafe_code)]
mod args;
mod commands;
use args::{Cli, Commands};
use clap::FromArgMatches;
use dtt_i18n::{AppLocale, CliMessage as M, Domain, Translator};

fn system_candidates() -> Vec<String> {
    #[cfg(not(windows))]
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(key) {
            if !value.is_empty() {
                return vec![if value == "C" || value == "POSIX" {
                    "en".into()
                } else {
                    value
                        .split('.')
                        .next()
                        .unwrap_or("en")
                        .split('@')
                        .next()
                        .unwrap_or("en")
                        .replace('_', "-")
                }];
            }
        }
    }
    sys_locale::get_locales().collect()
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(code) => std::process::ExitCode::from(code),
        Err(error) => {
            eprintln!("DTT: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
fn run() -> Result<u8, dtt_i18n::TranslationError> {
    let argv: Vec<_> = std::env::args_os().collect();
    let explicit = args::prefetch_locale(&argv);
    let environment = std::env::var_os("DTT_LOCALE");
    let system = system_candidates();
    let requested = explicit
        .as_ref()
        .or(environment.as_ref())
        .and_then(|value| value.to_str());
    let locale = requested
        .filter(|value| *value != "system")
        .and_then(AppLocale::matching)
        .unwrap_or_else(|| AppLocale::negotiate(system.iter().map(String::as_str)));
    let t = Translator::new(locale, Domain::Cli)?;
    let command = args::command(&t)?;
    if explicit.is_none()
        && environment.is_some()
        && requested.is_none_or(|value| value != "system" && AppLocale::matching(value).is_none())
    {
        eprintln!("{}", t.cli(M::InvalidEnvironmentLocale)?);
    }
    let matches = match command.clone().try_get_matches_from(&argv) {
        Ok(matches) => matches,
        Err(error) => {
            use clap::error::ErrorKind::*;
            if matches!(error.kind(), DisplayHelp | DisplayVersion) {
                print!("{error}");
                return Ok(0);
            }
            let label = match error.kind() {
                UnknownArgument | InvalidSubcommand => M::UnknownArgument,
                InvalidValue | ValueValidation | TooManyValues | TooFewValues
                | WrongNumberOfValues => M::InvalidValue,
                MissingRequiredArgument | MissingSubcommand => M::MissingArgument,
                ArgumentConflict => M::Conflict,
                _ => M::ParseError,
            };
            eprintln!("{}", t.cli(label)?);
            for (kind, value) in error.context() {
                use clap::error::ContextKind;
                if matches!(
                    kind,
                    ContextKind::InvalidArg
                        | ContextKind::InvalidValue
                        | ContextKind::ValidValue
                        | ContextKind::PriorArg
                ) {
                    eprintln!("  {value}");
                }
            }
            eprintln!("{}: dtt [--locale <LOCALE>] <COMMAND>", t.cli(M::Usage)?);
            if matches!(label, M::ParseError) {
                eprintln!("{}: {error}", t.cli(M::Technical)?);
            }
            return Ok(2);
        }
    };
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(_) => {
            eprintln!("{}", t.cli(M::ParseError)?);
            return Ok(2);
        }
    };
    if cli
        .locale
        .as_deref()
        .is_some_and(|value| value != "system" && AppLocale::matching(value).is_none())
    {
        eprintln!("{}", t.cli(M::InvalidLocale)?);
        return Ok(2);
    }
    let result = match cli.command {
        Commands::Generate(args) => commands::generate::run(args, locale, &t),
        Commands::DetectPaths => commands::detect_paths::run(&t),
    };
    match result {
        Ok(()) => Ok(0),
        Err(error) => {
            let errors = Translator::new(locale, Domain::Errors)?;
            use dtt_i18n::ErrorMessage as E;
            let code = match error.downcast_ref::<dtt_application::Error>() {
                Some(dtt_application::Error::Settings(issue)) => match issue {
                    dtt_application::SettingsIssue::Missing { .. } => E::MissingSetting,
                    dtt_application::SettingsIssue::InvalidPath { .. } => E::InvalidPath,
                    dtt_application::SettingsIssue::NoOutputLanguage => E::NoOutputLanguage,
                },
                Some(dtt_application::Error::NoTechnologyDefinitions) => E::NoTechnologyDefinitions,
                Some(
                    dtt_application::Error::ExecutablePath(_)
                    | dtt_application::Error::ExecutableParentMissing,
                ) => E::ExecutablePathUnavailable,
                Some(dtt_application::Error::SaveUnavailable(_)) => E::SaveUnavailable,
                Some(dtt_application::Error::UnsupportedBinarySave) => E::UnsupportedBinarySave,
                Some(dtt_application::Error::PlayerCountryMissing) => E::PlayerCountryMissing,
                Some(dtt_application::Error::PlayerCountryRequired) => E::PlayerCountryRequired,
                Some(dtt_application::Error::Cancelled) => E::GenerationCancelled,
                Some(dtt_application::Error::Stellaris(
                    dtt_application::StellarisError::Container(_),
                )) => E::SaveContainerCorrupt,
                Some(dtt_application::Error::Stellaris(
                    dtt_application::StellarisError::LauncherDb { .. },
                )) => E::LauncherDatabaseUnavailable,
                _ => E::Internal,
            };
            eprintln!(
                "{}\n{}: {error:#}",
                errors.error(code)?,
                t.cli(M::Technical)?
            );
            Ok(1)
        }
    }
}
