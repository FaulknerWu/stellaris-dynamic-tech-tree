#![forbid(unsafe_code)]

mod cancellation;
mod environment;
mod error;
mod generation;
mod progress;
mod save;

pub use cancellation::CancellationToken;
pub use dtt_core::condition::{ContextReason, UnknownConditionReason, UnknownStrategy};
pub use dtt_core::empire::Snapshot;
pub use dtt_core::technology::{Id as TechnologyId, SwapUnknownStrategy};
pub use dtt_stellaris::Error as StellarisError;
pub use dtt_stellaris::game_data::{
    DefinitionIssue, GameDataCategory, GameDataDiagnostic, GameDataDiagnosticKind,
};
pub use dtt_stellaris::save::SaveMetadata;
pub use environment::{
    DetectEnvironmentRequest, DetectedEnvironment, ResolveEnvironmentRequest, ResolvedEnvironment,
    detect_environment, output_directory, resolve_environment, resolve_generation_settings,
};
pub use error::{Error, Result};
pub use generation::{
    AppLocale, DefinitionFieldDiagnostic, GameLanguage, GenerationReport, GenerationSettings,
    GenerationSource, LanguageLocalisationDiagnostic, Presentation, ProgressCallback,
    ResolveGenerationSettingsRequest, RunGenerationRequest, RunGenerationResult, RunOutcome,
    SUPPORTED_OUTPUT_LANGUAGES, WriteFailure, WriteOperation, render_report, run_generation,
};
pub use progress::{GenerationStage, GenerationStatus};
pub use save::{
    InspectSaveRequest, InspectedSave, PlayerCountryCandidate, SaveAccount, SaveCampaign,
    SaveIndex, SaveIndexState, SaveLibrary, SaveLibrarySource, SaveScanDiagnostic,
    SaveScanFailureKind, SaveUnavailableReason, ScanSaveLibraryRequest, inspect_save,
    scan_save_library,
};

pub use dtt_stellaris::localisation::LocalisationFailureKind;

pub use error::SettingsIssue;
