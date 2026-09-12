#![forbid(unsafe_code)]

mod commands;
mod dto;
mod error;
mod state;

pub use dto::export_bindings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(state::DesktopState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::environment::get_bootstrap_data,
            commands::environment::resolve_environment,
            commands::save_library::scan_save_library,
            commands::save_library::inspect_save,
            commands::generation::run_generation,
            commands::generation::cancel_generation,
            commands::system::open_output_directory,
        ])
        .run(tauri::generate_context!())
        .expect("DTT 桌面应用启动失败");
}
