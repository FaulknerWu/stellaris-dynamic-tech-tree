use std::collections::HashMap;
use std::fs;
use std::path::Path;

use dtt_core::technology::Id;
use dtt_stellaris::output::{GameLanguage, WriteOutcome, WriteRequest, write};

fn generate(root: &Path, languages: &[GameLanguage], eligible: &[Id]) -> WriteOutcome {
    let trees = HashMap::from([(
        GameLanguage::English,
        HashMap::from([(Id::from("base"), "Tree\\nNext".into())]),
    )]);
    let descriptions = HashMap::from([(
        GameLanguage::English,
        HashMap::from([(Id::from("base"), "Original ".into())]),
    )]);
    write(&WriteRequest {
        eligible,
        render_results_by_language: &trees,
        original_descriptions_by_language: &descriptions,
        tiers: &HashMap::from([(Id::from("base"), 2)]),
        display_ids: &HashMap::from([(Id::from("base"), Id::from("variant"))]),
        languages,
        output_root_dir: root,
        report_body: "synthetic report\n",
    })
}

fn read_localisation(root: &Path, path: &str) -> String {
    let bytes = fs::read(root.join(path)).unwrap();
    assert_eq!(&bytes[..3], &[0xef, 0xbb, 0xbf]);
    String::from_utf8(bytes[3..].to_vec()).unwrap()
}

#[test]
fn output_has_bom_active_ids_original_descriptions_and_a_plain_text_report() {
    let temp = tempfile::tempdir().unwrap();
    let result = generate(temp.path(), &[GameLanguage::English], &["base".into()]);
    assert!(result.complete, "{:?}", result.failed);
    assert_eq!(result.written.len(), 3);
    let main = read_localisation(
        temp.path(),
        "localisation/english/zztechtreemain_l_english.yml",
    );
    assert!(main.starts_with("l_english:\n"));
    assert!(main.contains(" variant_techtree:0 \"Tree\\nNext\""));
    assert!(!main.contains("base_techtree"));
    let replaced = read_localisation(
        temp.path(),
        "localisation/english/replace/zztechtreereplaced_l_english.yml",
    );
    assert_eq!(
        replaced,
        "l_english:\n variant_desc:0 \"Original (Tier: 2)$variant_techtree$\""
    );
    let report = temp.path().join("dtt-save-report.txt");
    assert_eq!(fs::read(&report).unwrap(), b"synthetic report\n");
    assert_eq!(
        result.report_path.as_deref(),
        Some(report.to_str().unwrap())
    );
}

#[test]
fn switching_languages_removes_only_owned_files_and_preserves_user_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    assert!(
        generate(
            root,
            &[GameLanguage::English, GameLanguage::SimpChinese],
            &["base".into()]
        )
        .complete
    );
    let user = root.join("localisation/simp_chinese/user_l_simp_chinese.yml");
    let unknown = root.join("localisation/simp_chinese/replace/zztechtree_custom.yml");
    fs::write(&user, "user content").unwrap();
    fs::write(&unknown, "keep this too").unwrap();
    let result = generate(root, &[GameLanguage::English], &["base".into()]);
    assert!(result.complete, "{:?}", result.failed);
    assert_eq!(result.removed.len(), 2);
    assert!(
        !root
            .join("localisation/simp_chinese/zztechtreemain_l_simp_chinese.yml")
            .exists()
    );
    assert!(
        !root
            .join("localisation/simp_chinese/replace/zztechtreereplaced_l_simp_chinese.yml")
            .exists()
    );
    assert_eq!(fs::read_to_string(user).unwrap(), "user content");
    assert_eq!(fs::read_to_string(unknown).unwrap(), "keep this too");
}

#[test]
fn a_blocked_language_directory_reports_partial_failure_and_still_writes_report() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("localisation"), "not a directory").unwrap();
    let result = generate(temp.path(), &[GameLanguage::English], &["base".into()]);
    assert!(!result.complete);
    assert!(!result.failed.is_empty());
    assert_eq!(result.written.len(), 1);
    assert!(result.report_path.is_some());
    assert_eq!(
        fs::read_to_string(temp.path().join("localisation")).unwrap(),
        "not a directory"
    );
}

#[test]
fn repeated_output_is_byte_identical_regardless_of_eligible_input_order() {
    let temp = tempfile::tempdir().unwrap();
    let paths = [
        "localisation/english/zztechtreemain_l_english.yml",
        "localisation/english/replace/zztechtreereplaced_l_english.yml",
        "dtt-save-report.txt",
    ];
    assert!(
        generate(
            temp.path(),
            &[GameLanguage::English],
            &["z".into(), "base".into()]
        )
        .complete
    );
    let before: Vec<_> = paths
        .iter()
        .map(|path| fs::read(temp.path().join(path)).unwrap())
        .collect();
    assert!(
        generate(
            temp.path(),
            &[GameLanguage::English],
            &["base".into(), "z".into()]
        )
        .complete
    );
    let after: Vec<_> = paths
        .iter()
        .map(|path| fs::read(temp.path().join(path)).unwrap())
        .collect();
    assert_eq!(before, after);
}

#[test]
fn both_output_languages_preserve_markup_and_escape_once() {
    let temp = tempfile::tempdir().unwrap();
    let id = Id::from("base");
    let value =
        "quote \"raw\" and \\\"escaped\\\"\n$tech_id$ §H £physics£ path C:\\mods\\x literal \\n";
    let languages = [GameLanguage::English, GameLanguage::SimpChinese];
    let descriptions = languages
        .into_iter()
        .map(|language| (language, HashMap::from([(id.clone(), value.into())])))
        .collect();
    let result = write(&WriteRequest {
        eligible: &[id.clone()],
        render_results_by_language: &HashMap::new(),
        original_descriptions_by_language: &descriptions,
        tiers: &HashMap::new(),
        display_ids: &HashMap::new(),
        languages: &languages,
        output_root_dir: temp.path(),
        report_body: "report",
    });
    assert!(result.complete);
    for language in languages {
        let code = language.code();
        let output = read_localisation(
            temp.path(),
            &format!("localisation/{code}/replace/zztechtreereplaced_l_{code}.yml"),
        );
        assert!(output.starts_with(&format!("l_{code}:\n")));
        assert!(!output.contains(['\r', '\u{2068}', '\u{2069}']));
        assert!(output.contains("$tech_id$ §H £physics£"));
        assert!(
            output.contains("quote \\\"raw\\\" and \\\"escaped\\\"\\n"),
            "{output}"
        );
        assert!(output.contains("literal \\n"));
    }
}
#[test]
fn legacy_language_cleanup_keeps_unowned_files() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("localisation/french");
    fs::create_dir_all(dir.join("replace")).unwrap();
    fs::write(dir.join("zztechtreemain_l_french.yml"), "legacy").unwrap();
    fs::write(dir.join("user.yml"), "user").unwrap();
    let result = generate(temp.path(), &[GameLanguage::English], &[]);
    assert!(result.complete);
    assert_eq!(result.removed.len(), 1);
    assert_eq!(fs::read_to_string(dir.join("user.yml")).unwrap(), "user");
}
