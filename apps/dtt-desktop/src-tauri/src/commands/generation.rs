use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::ipc::Channel;

use crate::dto::{GenerationProgressDto, GenerationRequestDto, GenerationResultDto};
use crate::error::AppError;
use crate::state::{ActiveGeneration, DesktopState};

struct ActiveGenerationGuard<'state> {
    state: &'state DesktopState,
    id: u64,
}

impl Drop for ActiveGenerationGuard<'_> {
    fn drop(&mut self) {
        let Ok(mut active_generation) = self.state.active_generation.lock() else {
            return;
        };
        if active_generation
            .as_ref()
            .is_some_and(|generation| generation.id == self.id)
        {
            *active_generation = None;
        }
    }
}

#[tauri::command]
pub async fn run_generation(
    request: GenerationRequestDto,
    on_progress: Channel<GenerationProgressDto>,
    state: tauri::State<'_, DesktopState>,
) -> Result<GenerationResultDto, AppError> {
    let generation_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::internal(error.to_string()))?
        .as_nanos()
        .min(u128::from(u64::MAX)) as u64;
    let cancellation_token = dtt_application::CancellationToken::default();
    {
        let mut active_generation = state
            .active_generation
            .lock()
            .map_err(|_| AppError::internal("generation state lock poisoned"))?;
        if active_generation.is_some() {
            return Err(AppError::generation_busy());
        }
        *active_generation = Some(ActiveGeneration {
            id: generation_id,
            cancellation_token: cancellation_token.clone(),
        });
    }
    let _guard = ActiveGenerationGuard {
        state: state.inner(),
        id: generation_id,
    };

    let progress_callback: dtt_application::ProgressCallback = Arc::new(move |stage| {
        let _ = on_progress.send(GenerationProgressDto::StageChanged {
            stage: stage.into(),
            percent: stage.percent(),
        });
    });
    let application_request = dtt_application::RunGenerationRequest {
        source: dtt_application::GenerationSource::Save {
            path: request.save_file.into(),
            country_id: request.country_id,
        },
        settings: request.settings.into(),
        presentation: dtt_application::Presentation {
            report_locale: request.report_locale.into(),
        },
        render_limits: Default::default(),
        progress: Some(progress_callback),
        cancellation: cancellation_token,
    };

    tauri::async_runtime::spawn_blocking(move || {
        dtt_application::run_generation(&application_request)
            .map(Into::into)
            .map_err(Into::into)
    })
    .await
    .map_err(|error| AppError::internal(error.to_string()))?
}

#[tauri::command]
pub fn cancel_generation(state: tauri::State<'_, DesktopState>) -> Result<bool, AppError> {
    let active_generation = state
        .active_generation
        .lock()
        .map_err(|_| AppError::internal("generation state lock poisoned"))?;
    let Some(active_generation) = active_generation.as_ref() else {
        return Ok(false);
    };
    active_generation.cancellation_token.cancel();
    Ok(true)
}
