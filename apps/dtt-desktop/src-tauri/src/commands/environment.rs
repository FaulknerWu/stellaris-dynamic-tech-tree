use crate::dto::{BootstrapDataDto, EnvironmentDto, ResolveEnvironmentRequestDto};
use crate::error::AppError;

#[tauri::command]
pub fn get_bootstrap_data() -> Result<BootstrapDataDto, AppError> {
    let environment =
        dtt_application::detect_environment(&dtt_application::DetectEnvironmentRequest::default())
            .into();
    let environment_error = dtt_application::resolve_environment(
        &dtt_application::ResolveEnvironmentRequest::default(),
    )
    .err()
    .map(AppError::from);
    let output_directory = dtt_application::output_directory()
        .map_err(AppError::from)?
        .to_string_lossy()
        .into_owned();

    Ok(BootstrapDataDto {
        environment,
        environment_error,
        output_directory,
    })
}

#[tauri::command]
pub fn resolve_environment(
    request: ResolveEnvironmentRequestDto,
) -> Result<EnvironmentDto, AppError> {
    dtt_application::resolve_environment(&request.into())
        .map(Into::into)
        .map_err(Into::into)
}
