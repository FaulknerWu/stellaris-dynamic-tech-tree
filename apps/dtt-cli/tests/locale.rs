use std::process::{Command, Output};
fn run(args: &[&str], locale: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dtt"))
        .args(args)
        .env("DTT_LOCALE", locale)
        .output()
        .unwrap()
}
#[test]
fn help_resolves_locale_before_or_after_help_and_subcommand() {
    for args in [
        vec!["--locale", "zh-Hans", "--help"],
        vec!["--help", "--locale=zh-Hans"],
        vec!["generate", "--help", "--locale", "zh-Hans"],
    ] {
        let output = run(&args, "en");
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("用法"), "{text}");
        assert!(
            !text.contains("Usage:")
                && !text.contains("[default:")
                && !text.contains("[possible values:"),
            "{text}"
        );
        assert!(output.stderr.is_empty());
    }
}
#[test]
fn argument_errors_are_localised_and_use_stderr() {
    for args in [
        vec!["--locale=zh-Hans", "generate"],
        vec!["--locale=zh-Hans", "--wat"],
        vec![
            "generate",
            "--locale=zh-Hans",
            "save.sav",
            "--language=french",
        ],
        vec!["--locale=en", "--locale=zh-Hans", "detect-paths"],
        vec!["--locale=en", "generate", "--locale=zh-Hans", "save.sav"],
    ] {
        let output = run(&args, "en");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
    let output = run(&["--locale=zh-Hans", "--wat"], "en");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("未知参数")
    );
}
#[test]
fn invalid_environment_warns_once_but_explicit_system_skips_it() {
    let output = run(&["--help"], "invalid_locale");
    assert!(output.status.success());
    assert!(!output.stderr.is_empty());
    let output = run(&["--locale=system", "--help"], "invalid_locale");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}
#[test]
fn english_help_and_version_have_success_exit_codes() {
    let help = run(&["--help"], "en");
    assert!(help.status.success());
    assert!(String::from_utf8(help.stdout).unwrap().contains("Usage"));
    let version = run(&["--version"], "en");
    assert!(version.status.success());
    assert!(
        String::from_utf8(version.stdout)
            .unwrap()
            .starts_with("dtt ")
    );
}
