use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use atomic_write_file::AtomicWriteFile;
use dtt_core::technology::Id;

use super::{SupportedLanguage, WriteOutcome, WriteRequest};

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

        let main_body = build_main_for_lang(
            *language,
            &sorted_eligible,
            input.render_results_by_language.get(language),
            input.display_ids,
        );
        write_yml(&main_path, &main_body, &mut outcome);

        let replaced_body = build_replaced_for_lang(
            *language,
            &sorted_eligible,
            input.original_descriptions_by_language.get(language),
            input.tiers,
            input.display_ids,
        );
        write_yml(&replaced_path, &replaced_body, &mut outcome);
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
            .push(format!("{}: {e}", report_path.display())),
    }

    outcome.complete = outcome.failed.is_empty();
    outcome
}

fn build_main_for_lang(
    language: SupportedLanguage,
    eligible: &[Id],
    render_results: Option<&HashMap<Id, String>>,
    display_ids: &HashMap<Id, Id>,
) -> String {
    let lang = language.code();
    let strings = language.strings();
    let mut lines: Vec<String> = Vec::with_capacity(eligible.len() + 3);
    lines.push(format!("l_{lang}:"));
    lines.push(format!(" technology_tree_title:0 \"{}\"", strings.title));
    lines.push(format!(" tech_tree_max_level:0 \"{}\"", strings.top_level));

    for id in eligible {
        let tree = render_results
            .and_then(|results| results.get(id))
            .map(String::as_str)
            .unwrap_or("");
        let active = active_id_of(id, display_ids);
        lines.push(format!(" {active}_techtree:0 \"{tree}\""));
    }

    lines.join("\n")
}

fn build_replaced_for_lang(
    language: SupportedLanguage,
    eligible: &[Id],
    original_descriptions: Option<&HashMap<Id, String>>,
    tiers: &HashMap<Id, i32>,
    display_ids: &HashMap<Id, Id>,
) -> String {
    let lang = language.code();
    let strings = language.strings();
    let mut lines: Vec<String> = Vec::with_capacity(eligible.len() + 1);
    lines.push(format!("l_{lang}:"));

    for id in eligible {
        let active = active_id_of(id, display_ids);
        let original = original_descriptions
            .and_then(|descriptions| descriptions.get(id))
            .map(String::as_str)
            .unwrap_or("");
        let tier = tiers.get(id).copied().unwrap_or(0);

        let value = if original.is_empty() {
            format!(
                "({label}{tier})${active}_techtree$",
                label = strings.tier_label
            )
        } else {
            format!(
                "{original}({label}{tier})${active}_techtree$",
                label = strings.tier_label
            )
        };
        lines.push(format!(" {active}_desc:0 \"{value}\""));
    }

    lines.join("\n")
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
        Err(e) => outcome.failed.push(format!("{}: {e}", path.display())),
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
    for language in SupportedLanguage::ALL {
        if input.languages.contains(&language) {
            continue;
        }
        let lang = language.code();
        let language_dir = input.output_root_dir.join(LOCALISATION_DIR_NAME).join(lang);
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
                Err(error) => outcome.failed.push(format!("{}: {error}", path.display())),
            }
        }
    }
}
