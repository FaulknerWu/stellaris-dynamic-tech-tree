const COMMANDS: &[&str] = &[
    "get_bootstrap_data",
    "resolve_environment",
    "scan_save_library",
    "inspect_save",
    "run_generation",
    "cancel_generation",
    "open_output_directory",
];

fn main() {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("无法生成 Tauri 构建上下文");
}
