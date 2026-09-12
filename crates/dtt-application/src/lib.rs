#![forbid(unsafe_code)]

mod cancellation;
mod environment;
mod error;
mod generation;
mod progress;
mod save;

pub use cancellation::CancellationToken;
pub use dtt_core::condition::UnknownStrategy;
pub use dtt_core::empire::Snapshot;
pub use dtt_core::technology::SwapUnknownStrategy;
pub use dtt_stellaris::Error as StellarisError;
pub use dtt_stellaris::game_data::{GameDataCategory, GameDataDiagnostic, GameDataDiagnosticKind};
pub use dtt_stellaris::save::SaveMetadata;
pub use environment::{
    DetectEnvironmentRequest, DetectedEnvironment, ResolveEnvironmentRequest, ResolvedEnvironment,
    detect_environment, output_directory, resolve_environment, resolve_generation_settings,
};
pub use error::{Error, Result};
pub use generation::{
    DefinitionFieldDiagnostic, GenerationReport, GenerationSettings, GenerationSource,
    LanguageLocalisationDiagnostic, ProgressCallback, ResolveGenerationSettingsRequest,
    RunGenerationRequest, RunGenerationResult, RunOutcome, SupportedLanguage, run_generation,
};
pub use progress::{GenerationStage, GenerationStatus};
pub use save::{
    InspectSaveRequest, InspectedSave, PlayerCountryCandidate, SaveAccount, SaveCampaign,
    SaveIndex, SaveIndexState, SaveLibrary, SaveLibrarySource, ScanSaveLibraryRequest,
    inspect_save, scan_save_library,
};
