mod environment;
mod generation;
mod save;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::{Config, TS};

pub use environment::{
    BootstrapDataDto, DetectedEnvironmentDto, EnvironmentDto, ResolveEnvironmentRequestDto,
};
pub use generation::{
    GameLanguageDto, GenerationDiagnosticItemDto, GenerationDiagnosticsDto, GenerationProgressDto,
    GenerationRequestDto, GenerationResultDto, GenerationSettingsDto, GenerationStageDto,
    GenerationStatusDto, SwapUnknownStrategyDto, UnknownStrategyDto,
};
pub use save::{
    GovernmentDto, InspectSaveRequestDto, InspectedSaveDto, PlayerCountryCandidateDto,
    SaveAccountDto, SaveCampaignDto, SaveIndexDto, SaveIndexStateDto, SaveLibraryDto,
    SaveLibrarySourceDto, SaveMetadataDto, ScanSaveLibraryRequestDto, SnapshotDto, SpeciesDto,
};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ErrorContextDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub path: Option<String>,
}

pub fn export_bindings() -> Result<(), Box<dyn std::error::Error>> {
    let bindings_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/ipc/bindings");
    std::fs::create_dir_all(&bindings_dir)?;
    let config = Config::new()
        .with_out_dir(&bindings_dir)
        .with_large_int("number");

    BootstrapDataDto::export_all(&config)?;
    DetectedEnvironmentDto::export_all(&config)?;
    EnvironmentDto::export_all(&config)?;
    ResolveEnvironmentRequestDto::export_all(&config)?;
    GameLanguageDto::export_all(&config)?;
    GenerationProgressDto::export_all(&config)?;
    GenerationRequestDto::export_all(&config)?;
    GenerationResultDto::export_all(&config)?;
    GenerationDiagnosticsDto::export_all(&config)?;
    GenerationDiagnosticItemDto::export_all(&config)?;
    GenerationSettingsDto::export_all(&config)?;
    GenerationStageDto::export_all(&config)?;
    GenerationStatusDto::export_all(&config)?;
    SwapUnknownStrategyDto::export_all(&config)?;
    UnknownStrategyDto::export_all(&config)?;
    InspectSaveRequestDto::export_all(&config)?;
    InspectedSaveDto::export_all(&config)?;
    PlayerCountryCandidateDto::export_all(&config)?;
    SaveAccountDto::export_all(&config)?;
    SaveCampaignDto::export_all(&config)?;
    SaveIndexDto::export_all(&config)?;
    SaveIndexStateDto::export_all(&config)?;
    SaveLibraryDto::export_all(&config)?;
    SaveLibrarySourceDto::export_all(&config)?;
    SaveMetadataDto::export_all(&config)?;
    ScanSaveLibraryRequestDto::export_all(&config)?;
    SnapshotDto::export_all(&config)?;
    SpeciesDto::export_all(&config)?;
    GovernmentDto::export_all(&config)?;
    ErrorContextDto::export_all(&config)?;
    crate::error::AppError::export_all(&config)?;
    crate::error::ErrorCode::export_all(&config)?;
    Ok(())
}
