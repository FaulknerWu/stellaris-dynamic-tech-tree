use crate::dto::{
    InspectSaveRequestDto, InspectedSaveDto, SaveLibraryDto, ScanSaveLibraryRequestDto,
};
use crate::error::AppError;

#[tauri::command]
pub async fn scan_save_library(
    request: ScanSaveLibraryRequestDto,
) -> Result<SaveLibraryDto, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let application_request = dtt_application::ScanSaveLibraryRequest {
            source: request.source.into(),
            documents_dir: request.documents_dir.map(Into::into),
            steam_libraries: request
                .steam_libraries
                .into_iter()
                .map(Into::into)
                .collect(),
        };
        dtt_application::scan_save_library(&application_request)
            .map(Into::into)
            .map_err(Into::into)
    })
    .await
    .map_err(|error| AppError::internal(error.to_string()))?
}

#[tauri::command]
pub async fn inspect_save(request: InspectSaveRequestDto) -> Result<InspectedSaveDto, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        dtt_application::inspect_save(&dtt_application::InspectSaveRequest {
            save_file: request.save_file.into(),
            country_id: request.country_id,
        })
        .map(Into::into)
        .map_err(Into::into)
    })
    .await
    .map_err(|error| AppError::internal(error.to_string()))?
}
