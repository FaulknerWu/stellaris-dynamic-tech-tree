use std::collections::HashMap;
use std::path::Path;

use dtt_core::technology::Id;
use serde::{Deserialize, Serialize};

mod render;
mod writer;

pub use render::render_tree_content;
pub use writer::write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameLanguage {
    English,
    SimpChinese,
}

impl GameLanguage {
    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "english",
            Self::SimpChinese => "simp_chinese",
        }
    }

    pub const fn locale(self) -> dtt_i18n::AppLocale {
        match self {
            Self::English => dtt_i18n::AppLocale::En,
            Self::SimpChinese => dtt_i18n::AppLocale::ZhHans,
        }
    }
}

impl std::fmt::Display for GameLanguage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::str::FromStr for GameLanguage {
    type Err = UnsupportedLanguageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        SUPPORTED_OUTPUT_LANGUAGES
            .into_iter()
            .find(|language| language.code() == value)
            .ok_or_else(|| UnsupportedLanguageError(value.to_string()))
    }
}

pub const SUPPORTED_OUTPUT_LANGUAGES: [GameLanguage; 2] =
    [GameLanguage::English, GameLanguage::SimpChinese];

#[derive(Debug, Clone, thiserror::Error)]
#[error("unsupported output language `{0}`")]
pub struct UnsupportedLanguageError(String);

#[derive(Debug, Default, Clone)]
pub struct WriteOutcome {
    pub complete: bool,
    pub written: Vec<String>,
    pub removed: Vec<String>,
    pub failed: Vec<WriteFailure>,
    pub report_path: Option<String>,
}

pub struct WriteRequest<'a> {
    pub eligible: &'a [Id],
    pub render_results_by_language: &'a HashMap<GameLanguage, HashMap<Id, String>>,
    pub original_descriptions_by_language: &'a HashMap<GameLanguage, HashMap<Id, String>>,
    pub tiers: &'a HashMap<Id, i32>,
    pub display_ids: &'a HashMap<Id, Id>,
    pub languages: &'a [GameLanguage],
    pub output_root_dir: &'a Path,
    pub report_body: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct WriteFailure {
    pub path: String,
    pub operation: WriteOperation,
    pub technical_detail: String,
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteOperation {
    Write,
    Remove,
    Format,
}
impl WriteFailure {
    pub fn new(path: &Path, operation: WriteOperation, error: impl std::fmt::Display) -> Self {
        Self {
            path: path.to_string_lossy().into_owned(),
            operation,
            technical_detail: error.to_string(),
        }
    }
}
