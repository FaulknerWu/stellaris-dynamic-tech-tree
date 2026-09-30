use std::fs;
use std::io::Write;
use std::process::{Command, Output};

fn generate(block_output: bool) -> (tempfile::TempDir, Output) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let game = root.join("game");
    let documents = root.join("documents");
    fs::create_dir_all(game.join("common/technology")).unwrap();
    fs::create_dir_all(game.join("localisation")).unwrap();
    fs::create_dir_all(&documents).unwrap();
    fs::write(
        game.join("common/technology/test.txt"),
        "probe = { area = physics tier = 1 }",
    )
    .unwrap();
    let database = documents.join("launcher-v2.sqlite");
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute_batch("CREATE TABLE playsets (id TEXT, name TEXT, isActive INTEGER);")
        .unwrap();
    drop(connection);
    let save = root.join("synthetic.sav");
    let mut archive = zip::ZipWriter::new(fs::File::create(&save).unwrap());
    for (name, text) in [
        ("meta", "name = synthetic date = 2200.01.01"),
        (
            "gamestate",
            "player = { { country = 1 } } country = { 1 = {} }",
        ),
    ] {
        archive
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(text.as_bytes()).unwrap();
    }
    archive.finish().unwrap();
    let executable = root.join(format!("dtt{}", std::env::consts::EXE_SUFFIX));
    fs::copy(env!("CARGO_BIN_EXE_dtt"), &executable).unwrap();
    if block_output {
        fs::write(root.join("localisation"), "blocks directory creation").unwrap();
    }
    let output = Command::new(executable)
        .args(["--locale", "en", "generate"])
        .arg(save)
        .arg("--stellaris-root")
        .arg(game)
        .arg("--documents-dir")
        .arg(documents)
        .arg("--launcher-db")
        .arg(database)
        .output()
        .unwrap();
    (directory, output)
}

#[test]
fn incomplete_output_returns_a_failure_exit_code_and_keeps_diagnostics() {
    let (directory, output) = generate(true);
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generation incomplete"));
    assert!(stdout.contains("Failed operations:"));
    assert!(directory.path().join("dtt-save-report.txt").is_file());
    assert_eq!(
        fs::read_to_string(directory.path().join("localisation")).unwrap(),
        "blocks directory creation"
    );
}

#[test]
fn complete_output_returns_success_and_writes_both_localisation_files() {
    let (directory, output) = generate(false);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for relative in [
        "localisation/english/zztechtreemain_l_english.yml",
        "localisation/english/replace/zztechtreereplaced_l_english.yml",
    ] {
        assert!(
            fs::read(directory.path().join(relative))
                .unwrap()
                .starts_with(&[0xef, 0xbb, 0xbf])
        );
    }
}
