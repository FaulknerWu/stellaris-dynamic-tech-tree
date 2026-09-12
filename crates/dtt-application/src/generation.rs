mod pipeline;
mod report;

use std::path::PathBuf;
use std::sync::Arc;

use dtt_core::condition::UnknownStrategy;
use dtt_core::empire::Snapshot;
use dtt_core::render::RenderLimits;
use dtt_core::technology::SwapUnknownStrategy;
use serde::{Deserialize, Serialize};

use crate::cancellation::CancellationToken;
use crate::error::{Error, Result};
use crate::progress::{GenerationStage, GenerationStatus};

pub use dtt_stellaris::output::SupportedLanguage;
pub use pipeline::run_generation;
pub use report::{DefinitionFieldDiagnostic, GenerationReport, LanguageLocalisationDiagnostic};

pub type ProgressCallback = Arc<dyn Fn(GenerationStage) + Send + Sync>;

#[derive(Clone)]
pub struct RunGenerationRequest {
    pub source: GenerationSource,
    pub settings: GenerationSettings,
    pub render_limits: RenderLimits,
    pub progress: Option<ProgressCallback>,
    pub cancellation: CancellationToken,
}

#[derive(Debug, Clone)]
pub enum GenerationSource {
    Save {
        path: PathBuf,
        country_id: Option<i64>,
    },
    Snapshot(Box<Snapshot>),
}

impl RunGenerationRequest {
    pub fn from_save(save_path: impl Into<PathBuf>, settings: GenerationSettings) -> Self {
        Self {
            source: GenerationSource::Save {
                path: save_path.into(),
                country_id: None,
            },
            settings,
            render_limits: RenderLimits::default(),
            progress: None,
            cancellation: CancellationToken::default(),
        }
    }

    pub fn from_snapshot(snapshot: Snapshot, settings: GenerationSettings) -> Self {
        Self {
            source: GenerationSource::Snapshot(Box::new(snapshot)),
            settings,
            render_limits: RenderLimits::default(),
            progress: None,
            cancellation: CancellationToken::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RunGenerationResult {
    pub source_count: usize,
    pub technology_count: usize,
    pub output: RunOutcome,
    pub report: GenerationReport,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunOutcome {
    pub status: GenerationStatus,
    pub written: Vec<String>,
    pub removed: Vec<String>,
    pub failed: Vec<String>,
    pub report_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolveGenerationSettingsRequest {
    pub stellaris_root: Option<PathBuf>,
    pub documents_dir: Option<PathBuf>,
    pub launcher_db: Option<PathBuf>,
    pub languages: Vec<SupportedLanguage>,
    pub unknown_strategy: UnknownStrategy,
    pub swap_unknown_strategy: SwapUnknownStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSettings {
    pub stellaris_root: String,
    pub launcher_db: String,
    pub languages: Vec<SupportedLanguage>,
    #[serde(default)]
    pub unknown_strategy: UnknownStrategy,
    #[serde(default)]
    pub swap_unknown_strategy: SwapUnknownStrategy,
}

impl GenerationSettings {
    pub fn validate(&self) -> Result<()> {
        if self.stellaris_root.trim().is_empty() {
            return Err(Error::Settings("stellaris_root must not be empty".into()));
        }
        if self.languages.is_empty() {
            return Err(Error::Settings(
                "At least one output language is required (languages is empty)".into(),
            ));
        }
        if self.launcher_db.trim().is_empty() {
            return Err(Error::Settings("launcher_db must not be empty".into()));
        }
        Ok(())
    }
}
