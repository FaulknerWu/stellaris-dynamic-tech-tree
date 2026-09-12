use dtt_application as application;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SupportedLanguageDto {
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

impl From<SupportedLanguageDto> for application::SupportedLanguage {
    fn from(value: SupportedLanguageDto) -> Self {
        match value {
            SupportedLanguageDto::English => Self::English,
            SupportedLanguageDto::SimpChinese => Self::SimpChinese,
            SupportedLanguageDto::French => Self::French,
            SupportedLanguageDto::German => Self::German,
            SupportedLanguageDto::Spanish => Self::Spanish,
            SupportedLanguageDto::Russian => Self::Russian,
            SupportedLanguageDto::Korean => Self::Korean,
            SupportedLanguageDto::Japanese => Self::Japanese,
            SupportedLanguageDto::Polish => Self::Polish,
            SupportedLanguageDto::BrazPor => Self::BrazPor,
        }
    }
}

impl From<application::SupportedLanguage> for SupportedLanguageDto {
    fn from(value: application::SupportedLanguage) -> Self {
        match value {
            application::SupportedLanguage::English => Self::English,
            application::SupportedLanguage::SimpChinese => Self::SimpChinese,
            application::SupportedLanguage::French => Self::French,
            application::SupportedLanguage::German => Self::German,
            application::SupportedLanguage::Spanish => Self::Spanish,
            application::SupportedLanguage::Russian => Self::Russian,
            application::SupportedLanguage::Korean => Self::Korean,
            application::SupportedLanguage::Japanese => Self::Japanese,
            application::SupportedLanguage::Polish => Self::Polish,
            application::SupportedLanguage::BrazPor => Self::BrazPor,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum UnknownStrategyDto {
    IncludeFlagged,
    ExcludeStrict,
    Error,
}

impl From<UnknownStrategyDto> for application::UnknownStrategy {
    fn from(value: UnknownStrategyDto) -> Self {
        match value {
            UnknownStrategyDto::IncludeFlagged => Self::IncludeFlagged,
            UnknownStrategyDto::ExcludeStrict => Self::ExcludeStrict,
            UnknownStrategyDto::Error => Self::Error,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SwapUnknownStrategyDto {
    KeepBase,
    Error,
}

impl From<SwapUnknownStrategyDto> for application::SwapUnknownStrategy {
    fn from(value: SwapUnknownStrategyDto) -> Self {
        match value {
            SwapUnknownStrategyDto::KeepBase => Self::KeepBase,
            SwapUnknownStrategyDto::Error => Self::Error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GenerationSettingsDto {
    pub stellaris_root: String,
    pub launcher_db: String,
    pub languages: Vec<SupportedLanguageDto>,
    pub unknown_strategy: UnknownStrategyDto,
    pub swap_unknown_strategy: SwapUnknownStrategyDto,
}

impl From<GenerationSettingsDto> for application::GenerationSettings {
    fn from(value: GenerationSettingsDto) -> Self {
        Self {
            stellaris_root: value.stellaris_root,
            launcher_db: value.launcher_db,
            languages: value.languages.into_iter().map(Into::into).collect(),
            unknown_strategy: value.unknown_strategy.into(),
            swap_unknown_strategy: value.swap_unknown_strategy.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GenerationRequestDto {
    pub save_file: String,
    #[ts(optional)]
    pub country_id: Option<i64>,
    pub settings: GenerationSettingsDto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GenerationStageDto {
    SaveParse,
    LoadOrder,
    IngestTech,
    Relations,
    IngestL10n,
    Render,
    Cycles,
    WriteOutput,
    Done,
}

impl From<application::GenerationStage> for GenerationStageDto {
    fn from(value: application::GenerationStage) -> Self {
        match value {
            application::GenerationStage::SaveParse => Self::SaveParse,
            application::GenerationStage::LoadOrder => Self::LoadOrder,
            application::GenerationStage::IngestTech => Self::IngestTech,
            application::GenerationStage::Relations => Self::Relations,
            application::GenerationStage::IngestL10n => Self::IngestL10n,
            application::GenerationStage::Render => Self::Render,
            application::GenerationStage::Cycles => Self::Cycles,
            application::GenerationStage::WriteOutput => Self::WriteOutput,
            application::GenerationStage::Done => Self::Done,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GenerationProgressDto {
    StageChanged {
        stage: GenerationStageDto,
        percent: u8,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum GenerationStatusDto {
    Success,
    Incomplete,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GenerationResultDto {
    pub status: GenerationStatusDto,
    pub source_count: usize,
    pub technology_count: usize,
    pub eligible_count: usize,
    pub swap_matched: usize,
    pub swap_no_match: usize,
    pub swap_uncertain: usize,
    pub written: Vec<String>,
    pub removed: Vec<String>,
    pub failed: Vec<String>,
    #[ts(optional)]
    pub report_path: Option<String>,
    pub diagnostics: GenerationDiagnosticsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GenerationDiagnosticsDto {
    pub unknown_conditions: Vec<GenerationDiagnosticItemDto>,
    pub deferred_conditions: Vec<GenerationDiagnosticItemDto>,
    pub game_data: Vec<GenerationDiagnosticItemDto>,
    pub unhandled_definitions: Vec<GenerationDiagnosticItemDto>,
    pub localisation: Vec<GenerationDiagnosticItemDto>,
    pub cycles: Vec<GenerationDiagnosticItemDto>,
    pub write_failures: Vec<GenerationDiagnosticItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GenerationDiagnosticItemDto {
    pub summary: String,
    #[ts(optional)]
    pub detail: Option<String>,
}

impl From<application::RunGenerationResult> for GenerationResultDto {
    fn from(value: application::RunGenerationResult) -> Self {
        let status = match value.output.status {
            application::GenerationStatus::Success => GenerationStatusDto::Success,
            application::GenerationStatus::Incomplete => GenerationStatusDto::Incomplete,
        };
        let unknown_conditions = value
            .report
            .unknown_triggers
            .iter()
            .map(|diagnostic| GenerationDiagnosticItemDto {
                summary: diagnostic.reason.trigger_name().to_string(),
                detail: Some(format!(
                    "{}；出现 {} 次；影响科技：{}",
                    diagnostic.reason,
                    diagnostic.occurrences,
                    diagnostic
                        .tech_ids
                        .iter()
                        .map(|technology_id| technology_id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            })
            .collect();
        let deferred_conditions = value
            .report
            .deferred_triggers
            .iter()
            .map(|diagnostic| GenerationDiagnosticItemDto {
                summary: diagnostic.name.clone(),
                detail: Some(format!(
                    "出现 {} 次；影响科技：{}",
                    diagnostic.occurrences,
                    diagnostic
                        .tech_ids
                        .iter()
                        .map(|technology_id| technology_id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            })
            .collect();
        let game_data = value
            .report
            .game_data_diagnostics
            .iter()
            .map(|diagnostic| GenerationDiagnosticItemDto {
                summary: diagnostic
                    .subject
                    .clone()
                    .unwrap_or_else(|| diagnostic.source.clone()),
                detail: Some(
                    format!(
                        "{} · {} · {}{}",
                        diagnostic.source,
                        diagnostic.category.as_str(),
                        diagnostic.kind.as_str(),
                        diagnostic
                            .byte_offset
                            .map(|offset| format!(" · byte {offset}"))
                            .unwrap_or_default()
                    ) + &format!("\n{}", diagnostic.message),
                ),
            })
            .collect();
        let unhandled_definitions = value
            .report
            .unhandled_definition_fields
            .iter()
            .map(|diagnostic| GenerationDiagnosticItemDto {
                summary: diagnostic.technology_id.as_str().to_string(),
                detail: Some(format!(
                    "{}\n{}",
                    diagnostic.source,
                    diagnostic.fields.join(", ")
                )),
            })
            .collect();
        let localisation = value
            .report
            .localisation_diagnostics
            .iter()
            .map(|diagnostic| GenerationDiagnosticItemDto {
                summary: format!("{} · {}", diagnostic.language, diagnostic.source),
                detail: Some(format!("{}\n{}", diagnostic.path, diagnostic.message)),
            })
            .collect();
        let mut cycles = value
            .report
            .cycles_self_refs
            .iter()
            .map(|technology_id| GenerationDiagnosticItemDto {
                summary: technology_id.as_str().to_string(),
                detail: None,
            })
            .collect::<Vec<_>>();
        cycles.extend(value.report.cycles_complex.iter().map(|cycle| {
            GenerationDiagnosticItemDto {
                summary: cycle
                    .iter()
                    .map(|technology_id| technology_id.as_str())
                    .collect::<Vec<_>>()
                    .join(" → "),
                detail: None,
            }
        }));
        let write_failures = value
            .output
            .failed
            .iter()
            .map(|failure| GenerationDiagnosticItemDto {
                summary: failure.clone(),
                detail: None,
            })
            .collect();
        Self {
            status,
            source_count: value.source_count,
            technology_count: value.technology_count,
            eligible_count: value.report.eligible.len(),
            swap_matched: value.report.swap_matched,
            swap_no_match: value.report.swap_nomatch,
            swap_uncertain: value.report.swap_uncertain,
            written: value.output.written,
            removed: value.output.removed,
            failed: value.output.failed,
            report_path: value.output.report_path,
            diagnostics: GenerationDiagnosticsDto {
                unknown_conditions,
                deferred_conditions,
                game_data,
                unhandled_definitions,
                localisation,
                cycles,
                write_failures,
            },
        }
    }
}
