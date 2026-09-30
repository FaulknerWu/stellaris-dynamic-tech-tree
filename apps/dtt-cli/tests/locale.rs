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
    let cases: &[&[&str]] = &[
        &["--locale", "zh-Hans", "--help"],
        &["--help", "--locale=zh-Hans"],
        &["generate", "--help", "--locale", "zh-Hans"],
    ];
    for args in cases {
        let output = run(args, "en");
        assert!(output.status.success(), "{args:?}: {output:?}");
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
    let cases: &[(&[&str], &str)] = &[
        (&["--locale=zh-Hans", "generate"], "缺少必需参数或子命令"),
        (&["--locale=zh-Hans", "--wat"], "未知参数或子命令"),
        (
            &[
                "generate",
                "--locale=zh-Hans",
                "save.sav",
                "--language=french",
            ],
            "参数值无效",
        ),
        (
            &["--locale=en", "--locale=zh-Hans", "detect-paths"],
            "Conflicting or repeated arguments",
        ),
        (
            &["--locale=en", "generate", "--locale=zh-Hans", "save.sav"],
            "Invalid argument value",
        ),
    ];
    for (args, expected) in cases {
        let output = run(args, "en");
        assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{args:?}: {output:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(stderr.lines().next(), Some(*expected), "{args:?}: {stderr}");
    }
}

#[test]
fn invalid_environment_warns_once_but_explicit_system_skips_it() {
    let output = run(&["--help"], "invalid_locale");
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert_eq!(stderr.matches("DTT_LOCALE").count(), 1, "{stderr}");
    let output = run(&["--locale=system", "--help"], "invalid_locale");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

#[test]
fn japanese_and_russian_help_errors_and_environment_are_localised() {
    for (locale, usage, error) in [
        ("ja", "使用方法", "不明な引数またはサブコマンドです"),
        ("ru", "Использование", "Неизвестный аргумент или подкоманда"),
    ] {
        for args in [
            vec!["--locale", locale, "--help"],
            vec!["--help", "--locale", locale],
            vec!["generate", "--help", "--locale", locale],
        ] {
            let output = run(&args, "en");
            assert!(output.status.success(), "{output:?}");
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(text.contains(usage), "{text}");
            assert!(!text.contains("Usage:"), "{text}");
            assert!(output.stderr.is_empty());
        }
        let output = run(&["--help"], locale);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains(usage));
        assert!(output.stderr.is_empty());
        let output = run(&["--locale", locale, "--wat"], "en");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap().lines().next(),
            Some(error)
        );
        let output = run(&["--locale=en", "--help"], locale);
        assert!(String::from_utf8(output.stdout).unwrap().contains("Usage:"));
    }
}
