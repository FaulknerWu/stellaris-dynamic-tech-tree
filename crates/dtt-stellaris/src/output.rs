use std::collections::HashMap;
use std::path::Path;

use dtt_core::technology::Id;
use serde::{Deserialize, Serialize};

mod language;
mod render;
mod writer;

pub use language::LangStrings;
pub use render::render_tree_content;
pub use writer::write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportedLanguage {
    English,
    SimpChinese,
    French,
    German,
    Spanish,
    Russian,
    Korean,
    Japanese,
    Polish,
    BrazPor,
}

impl SupportedLanguage {
    pub const ALL: [Self; 10] = [
        Self::English,
        Self::SimpChinese,
        Self::French,
        Self::German,
        Self::Spanish,
        Self::Russian,
        Self::Korean,
        Self::Japanese,
        Self::Polish,
        Self::BrazPor,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "english",
            Self::SimpChinese => "simp_chinese",
            Self::French => "french",
            Self::German => "german",
            Self::Spanish => "spanish",
            Self::Russian => "russian",
            Self::Korean => "korean",
            Self::Japanese => "japanese",
            Self::Polish => "polish",
            Self::BrazPor => "braz_por",
        }
    }

    pub const fn strings(self) -> &'static LangStrings {
        language::strings_for(self)
    }
}

impl std::fmt::Display for SupportedLanguage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::str::FromStr for SupportedLanguage {
    type Err = UnsupportedLanguageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|language| language.code() == value)
            .ok_or_else(|| UnsupportedLanguageError(value.to_string()))
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("unsupported output language `{0}`")]
pub struct UnsupportedLanguageError(String);

#[derive(Debug, Default, Clone)]
pub struct WriteOutcome {
    pub complete: bool,
    pub written: Vec<String>,
    pub removed: Vec<String>,
    pub failed: Vec<String>,
    pub report_path: Option<String>,
}

pub struct WriteRequest<'a> {
    pub eligible: &'a [Id],
    pub render_results_by_language: &'a HashMap<SupportedLanguage, HashMap<Id, String>>,
    pub original_descriptions_by_language: &'a HashMap<SupportedLanguage, HashMap<Id, String>>,
    pub tiers: &'a HashMap<Id, i32>,
    pub display_ids: &'a HashMap<Id, Id>,
    pub languages: &'a [SupportedLanguage],
    pub output_root_dir: &'a Path,
    pub report_body: &'a str,
}
