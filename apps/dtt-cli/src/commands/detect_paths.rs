use std::path::Path;

use anyhow::Result;
use dtt_application::{DetectEnvironmentRequest, DetectedEnvironment, detect_environment};

pub fn run() -> Result<()> {
    let environment = detect_environment(&DetectEnvironmentRequest::default());
    print_environment(&environment);
    Ok(())
}

fn print_environment(environment: &DetectedEnvironment) {
    println!(
        "群星本体目录: {}",
        optional_path(environment.game_root.as_deref())
    );
    println!(
        "文档群星目录: {}",
        optional_path(environment.documents_dir.as_deref())
    );
    println!(
        "启动器数据库: {}",
        optional_path(environment.launcher_db.as_deref())
    );
    if !environment.steam_libraries.is_empty() {
        println!("Steam 库:");
        for library in &environment.steam_libraries {
            println!("  {}", library.display());
        }
    }
}

fn optional_path(path: Option<&Path>) -> String {
    path.map(|path| path.display().to_string())
        .unwrap_or_else(|| "未检测到".to_string())
}
