use tauri_plugin_opener::OpenerExt;

use crate::error::AppError;

#[tauri::command]
pub fn open_output_directory(app: tauri::AppHandle) -> Result<(), AppError> {
    let output_directory = dtt_application::output_directory().map_err(AppError::from)?;
    app.opener()
        .open_path(output_directory.to_string_lossy(), None::<&str>)
        .map_err(|error| AppError::internal(error.to_string()))
}
