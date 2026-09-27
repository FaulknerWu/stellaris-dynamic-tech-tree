use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use atomic_write_file::AtomicWriteFile;
use dtt_core::technology::Id;

use super::{GameLanguage, WriteFailure, WriteOperation, WriteOutcome, WriteRequest};
use dtt_i18n::{Domain, TranslationError, Translator};

const MAIN_LOC_TEMPLATE: &str = "zztechtreemain_l_{lang}.yml";
const REPLACED_LOC_TEMPLATE: &str = "zztechtreereplaced_l_{lang}.yml";
const REPORT_FILENAME: &str = "dtt-save-report.txt";
const LOCALISATION_DIR_NAME: &str = "localisation";
const REPLACE_DIR_NAME: &str = "replace";
const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

pub fn write(input: &WriteRequest) -> WriteOutcome {
    let mut outcome = WriteOutcome::default();
    let output_root = input.output_root_dir;

    let mut sorted_eligible: Vec<Id> = input.eligible.to_vec();
    sorted_eligible.sort();

    for language in input.languages {
        let lang = language.code();
        let language_dir = output_root.join(LOCALISATION_DIR_NAME).join(lang);
        let main_path = language_dir.join(MAIN_LOC_TEMPLATE.replace("{lang}", lang));
        let replaced_path = language_dir
            .join(REPLACE_DIR_NAME)
            .join(REPLACED_LOC_TEMPLATE.replace("{lang}", lang));

        let translator = match Translator::new(language.locale(), Domain::Game) {
            Ok(translator) => translator,
            Err(error) => {
                outcome
                    .failed
                    .push(WriteFailure::new(&main_path, WriteOperation::Format, error));
                continue;
            }
        };
        let main_body = build_main_for_lang(
            *language,
            &translator,
            &sorted_eligible,
            input.render_results_by_language.get(language),
            input.display_ids,
        );

        let replaced_body = build_replaced_for_lang(
            *language,
            &translator,
            &sorted_eligible,
            input.original_descriptions_by_language.get(language),
            input.tiers,
            input.display_ids,
        );
        match (main_body, replaced_body) {
            (Ok(main), Ok(replaced)) => {
                write_yml(&main_path, &main, &mut outcome);
                write_yml(&replaced_path, &replaced, &mut outcome);
            }
            (Err(error), _) | (_, Err(error)) => {
                outcome
                    .failed
                    .push(WriteFailure::new(&main_path, WriteOperation::Format, error))
            }
        }
    }

    cleanup_stale_language_files(input, &mut outcome);

    let report_path = output_root.join(REPORT_FILENAME);
    match write_bytes(&report_path, input.report_body.as_bytes()) {
        Ok(()) => {
            outcome
                .written
                .push(report_path.to_string_lossy().into_owned());
            outcome.report_path = Some(report_path.to_string_lossy().into_owned());
        }
        Err(e) => outcome
            .failed
            .push(WriteFailure::new(&report_path, WriteOperation::Write, e)),
    }

    outcome.complete = outcome.failed.is_empty();
    outcome
}

fn build_main_for_lang(
    language: GameLanguage,
    translator: &Translator,
    eligible: &[Id],
    render_results: Option<&HashMap<Id, String>>,
    display_ids: &HashMap<Id, Id>,
) -> Result<String, TranslationError> {
    let lang = language.code();
    let mut lines: Vec<String> = Vec::with_capacity(eligible.len() + 3);
    lines.push(format!("l_{lang}:"));
    lines.push(format!(
        " technology_tree_title:0 \"{}\"",
        escape_value(&translator.game_title()?)
    ));
    lines.push(format!(
        " tech_tree_max_level:0 \"{}\"",
        escape_value(&translator.game_max_level()?)
    ));

    for id in eligible {
        let tree = render_results
            .and_then(|results| results.get(id))
            .map(String::as_str)
            .unwrap_or("");
        let active = active_id_of(id, display_ids);
        let tree = escape_value(tree);
        lines.push(format!(" {active}_techtree:0 \"{tree}\""));
    }

    Ok(lines.join("\n"))
}

fn build_replaced_for_lang(
    language: GameLanguage,
    translator: &Translator,
    eligible: &[Id],
    original_descriptions: Option<&HashMap<Id, String>>,
    tiers: &HashMap<Id, i32>,
    display_ids: &HashMap<Id, Id>,
) -> Result<String, TranslationError> {
    let lang = language.code();
    let mut lines: Vec<String> = Vec::with_capacity(eligible.len() + 1);
    lines.push(format!("l_{lang}:"));

    for id in eligible {
        let active = active_id_of(id, display_ids);
        let original = original_descriptions
            .and_then(|descriptions| descriptions.get(id))
            .map(String::as_str)
            .unwrap_or("");
        let tier = tiers.get(id).copied().unwrap_or(0);

        let label = escape_value(&translator.game_tier(tier)?);
        let original = escape_value(original);
        let value = format!("{original}({label})${active}_techtree$");
        lines.push(format!(" {active}_desc:0 \"{value}\""));
    }

    Ok(lines.join("\n"))
}

fn active_id_of<'a>(id: &'a Id, display_ids: &'a HashMap<Id, Id>) -> &'a str {
    display_ids
        .get(id)
        .map(Id::as_str)
        .unwrap_or_else(|| id.as_str())
}

fn write_yml(path: &Path, body: &str, outcome: &mut WriteOutcome) {
    let mut bytes: Vec<u8> = Vec::with_capacity(body.len() + UTF8_BOM.len());
    bytes.extend_from_slice(UTF8_BOM);
    bytes.extend_from_slice(body.as_bytes());
    match write_bytes(path, &bytes) {
        Ok(()) => outcome.written.push(path.to_string_lossy().into_owned()),
        Err(e) => outcome
            .failed
            .push(WriteFailure::new(path, WriteOperation::Write, e)),
    }
}

fn write_bytes(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = AtomicWriteFile::open(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    file.commit()
}

fn cleanup_stale_language_files(input: &WriteRequest<'_>, outcome: &mut WriteOutcome) {
    let root = input.output_root_dir.join(LOCALISATION_DIR_NAME);
    let entries = match fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            outcome
                .failed
                .push(WriteFailure::new(&root, WriteOperation::Remove, error));
            return;
        }
    };
    // Ownership is determined by the generated filenames, independent of supported languages.
    let mut directories = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => match entry.file_type() {
                Ok(kind) if kind.is_dir() => directories.push(entry),
                Ok(_) => {}
                Err(error) => outcome.failed.push(WriteFailure::new(
                    &entry.path(),
                    WriteOperation::Remove,
                    error,
                )),
            },
            Err(error) => {
                outcome
                    .failed
                    .push(WriteFailure::new(&root, WriteOperation::Remove, error))
            }
        }
    }
    directories.sort_by_key(|entry| entry.file_name());
    for entry in directories {
        let name = entry.file_name();
        let Some(lang) = name.to_str() else {
            continue;
        };
        if input
            .languages
            .iter()
            .any(|language| language.code() == lang)
        {
            continue;
        }
        let language_dir = entry.path();
        let owned_paths = [
            language_dir.join(MAIN_LOC_TEMPLATE.replace("{lang}", lang)),
            language_dir
                .join(REPLACE_DIR_NAME)
                .join(REPLACED_LOC_TEMPLATE.replace("{lang}", lang)),
        ];
        for path in owned_paths {
            match fs::remove_file(&path) {
                Ok(()) => outcome.removed.push(path.to_string_lossy().into_owned()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    outcome
                        .failed
                        .push(WriteFailure::new(&path, WriteOperation::Remove, error))
                }
            }
        }
    }
}

// Clausewitz input already contains escaped sequences. Preserve each existing escape once.
fn escape_value(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                output.push('\\');
                if matches!(chars.peek(), Some('n' | 'r' | 't' | '"' | '\\')) {
                    if let Some(next) = chars.next() {
                        output.push(next);
                    }
                } else {
                    output.push('\\');
                }
            }
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => {}
            _ => output.push(ch),
        }
    }
    output
}
