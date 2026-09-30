use dtt_application as application;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
pub enum AppLocaleDto {
    #[serde(rename = "en")]
    En,
    #[serde(rename = "zh-Hans")]
    ZhHans,
    #[serde(rename = "ja")]
    Ja,
    #[serde(rename = "ru")]
    Ru,
}
impl From<AppLocaleDto> for application::AppLocale {
    fn from(value: AppLocaleDto) -> Self {
        match value {
            AppLocaleDto::En => Self::En,
            AppLocaleDto::ZhHans => Self::ZhHans,
            AppLocaleDto::Ja => Self::Ja,
            AppLocaleDto::Ru => Self::Ru,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum GameLanguageDto {
    English,
    SimpChinese,
    Japanese,
    Russian,
}

impl From<GameLanguageDto> for application::GameLanguage {
    fn from(value: GameLanguageDto) -> Self {
        match value {
            GameLanguageDto::English => Self::English,
            GameLanguageDto::SimpChinese => Self::SimpChinese,
            GameLanguageDto::Japanese => Self::Japanese,
            GameLanguageDto::Russian => Self::Russian,
        }
    }
}

impl From<application::GameLanguage> for GameLanguageDto {
    fn from(value: application::GameLanguage) -> Self {
        match value {
            application::GameLanguage::English => Self::English,
            application::GameLanguage::SimpChinese => Self::SimpChinese,
            application::GameLanguage::Japanese => Self::Japanese,
            application::GameLanguage::Russian => Self::Russian,
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
    pub languages: Vec<GameLanguageDto>,
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
    pub report_locale: AppLocaleDto,
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
    pub failed: Vec<GenerationDiagnosticItemDto>,
    #[ts(optional)]
    pub report_path: Option<String>,
    pub diagnostics: GenerationDiagnosticsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GenerationDiagnosticsDto {
    pub load_order: Vec<GenerationDiagnosticItemDto>,
    pub unknown_conditions: Vec<GenerationDiagnosticItemDto>,
    pub deferred_conditions: Vec<GenerationDiagnosticItemDto>,
    pub game_data: Vec<GenerationDiagnosticItemDto>,
    pub unhandled_definitions: Vec<GenerationDiagnosticItemDto>,
    pub localisation: Vec<GenerationDiagnosticItemDto>,
    pub cycles: Vec<GenerationDiagnosticItemDto>,
    pub write_failures: Vec<GenerationDiagnosticItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum GenerationDiagnosticItemDto {
    MissingModDescriptor {
        mod_name: String,
    },
    UnknownCondition {
        reason: UnknownConditionDto,
        name: String,
        occurrences: usize,
        technologies: Vec<String>,
    },
    DeferredCondition {
        name: String,
        occurrences: usize,
        technologies: Vec<String>,
    },
    GameData {
        source: String,
        category: String,
        diagnostic_kind: String,
        subject: Option<String>,
        byte_offset: Option<usize>,
        issue: DefinitionIssueDto,
        technical_detail: Option<String>,
    },
    UnhandledDefinition {
        technology: String,
        source: String,
        fields: Vec<String>,
    },
    Localisation {
        reason: LocalisationFailureKindDto,
        language: String,
        source: String,
        path: String,
        technical_detail: String,
    },
    SelfReference {
        technology: String,
    },
    Cycle {
        technologies: Vec<String>,
    },
    WriteFailed {
        path: String,
        operation: WriteOperationDto,
        technical_detail: String,
    },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum WriteOperationDto {
    Write,
    Remove,
    Format,
}
impl From<&application::WriteFailure> for GenerationDiagnosticItemDto {
    fn from(value: &application::WriteFailure) -> Self {
        Self::WriteFailed {
            path: value.path.clone(),
            operation: match value.operation {
                application::WriteOperation::Write => WriteOperationDto::Write,
                application::WriteOperation::Remove => WriteOperationDto::Remove,
                application::WriteOperation::Format => WriteOperationDto::Format,
            },
            technical_detail: value.technical_detail.clone(),
        }
    }
}
impl From<application::RunGenerationResult> for GenerationResultDto {
    fn from(value: application::RunGenerationResult) -> Self {
        use GenerationDiagnosticItemDto as D;
        let report = &value.report;
        let ids = |items: &[application::TechnologyId]| {
            items.iter().map(ToString::to_string).collect::<Vec<_>>()
        };
        let mut cycles = report
            .cycles_self_refs
            .iter()
            .map(|id| D::SelfReference {
                technology: id.to_string(),
            })
            .collect::<Vec<_>>();
        cycles.extend(report.cycles_complex.iter().map(|items| D::Cycle {
            technologies: ids(items),
        }));
        let failed = value.output.failed.iter().map(D::from).collect::<Vec<_>>();
        Self {
            status: match value.output.status {
                application::GenerationStatus::Success => GenerationStatusDto::Success,
                application::GenerationStatus::Incomplete => GenerationStatusDto::Incomplete,
            },
            source_count: value.source_count,
            technology_count: value.technology_count,
            eligible_count: report.eligible.len(),
            swap_matched: report.swap_matched,
            swap_no_match: report.swap_nomatch,
            swap_uncertain: report.swap_uncertain,
            written: value.output.written,
            removed: value.output.removed,
            failed: failed.clone(),
            report_path: value.output.report_path,
            diagnostics: GenerationDiagnosticsDto {
                load_order: report
                    .missing_mod_descriptors
                    .iter()
                    .map(|name| D::MissingModDescriptor {
                        mod_name: name.clone(),
                    })
                    .collect(),
                unknown_conditions: report
                    .unknown_triggers
                    .iter()
                    .map(|d| D::UnknownCondition {
                        reason: (&d.reason).into(),
                        name: d.reason.trigger_name().into(),
                        occurrences: d.occurrences,
                        technologies: ids(&d.tech_ids),
                    })
                    .collect(),
                deferred_conditions: report
                    .deferred_triggers
                    .iter()
                    .map(|d| D::DeferredCondition {
                        name: d.name.clone(),
                        occurrences: d.occurrences,
                        technologies: ids(&d.tech_ids),
                    })
                    .collect(),
                game_data: report
                    .game_data_diagnostics
                    .iter()
                    .map(|d| D::GameData {
                        source: d.source.clone(),
                        category: d.category.as_str().into(),
                        diagnostic_kind: d.kind.as_str().into(),
                        subject: d.subject.clone(),
                        byte_offset: d.byte_offset,
                        issue: (&d.issue).into(),
                        technical_detail: d.technical_detail.clone(),
                    })
                    .collect(),
                unhandled_definitions: report
                    .unhandled_definition_fields
                    .iter()
                    .map(|d| D::UnhandledDefinition {
                        technology: d.technology_id.to_string(),
                        source: d.source.clone(),
                        fields: d.fields.clone(),
                    })
                    .collect(),
                localisation: report
                    .localisation_diagnostics
                    .iter()
                    .map(|d| D::Localisation {
                        reason: d.kind.into(),
                        language: d.language.clone(),
                        source: d.source.clone(),
                        path: d.path.clone(),
                        technical_detail: d.technical_detail.clone(),
                    })
                    .collect(),
                cycles,
                write_failures: failed,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DefinitionIssueDto {
    InvalidUtf8,
    InvalidSyntax,
    ExpectedScalar,
    ExpectedObject,
    InvalidField { field: String },
    Overwritten { previous: Option<String> },
    InlineCycle { script: String },
    InlineMissing { script: String },
    InvalidInlineCall,
    IsolatedBranch,
    InvalidCondition,
    MalformedArgument,
    StructuredArgument,
    DuplicateParameter,
    InvalidSwap { index: usize, field: String },
    DuplicateField { field: String },
    FieldCase { field: String, expected: String },
}
impl From<&application::DefinitionIssue> for DefinitionIssueDto {
    fn from(issue: &application::DefinitionIssue) -> Self {
        use application::DefinitionIssue as I;
        match issue {
            I::InvalidUtf8 => Self::InvalidUtf8,
            I::InvalidSyntax => Self::InvalidSyntax,
            I::ExpectedScalar => Self::ExpectedScalar,
            I::ExpectedObject => Self::ExpectedObject,
            I::InvalidField { field } => Self::InvalidField {
                field: field.clone(),
            },
            I::Overwritten { previous } => Self::Overwritten {
                previous: previous.clone(),
            },
            I::InlineCycle { script } => Self::InlineCycle {
                script: script.clone(),
            },
            I::InlineMissing { script } => Self::InlineMissing {
                script: script.clone(),
            },
            I::InvalidInlineCall => Self::InvalidInlineCall,
            I::IsolatedBranch => Self::IsolatedBranch,
            I::InvalidCondition => Self::InvalidCondition,
            I::MalformedArgument => Self::MalformedArgument,
            I::StructuredArgument => Self::StructuredArgument,
            I::DuplicateParameter => Self::DuplicateParameter,
            I::InvalidSwap { index, field } => Self::InvalidSwap {
                index: *index,
                field: field.clone(),
            },
            I::DuplicateField { field } => Self::DuplicateField {
                field: field.clone(),
            },
            I::FieldCase { field, expected } => Self::FieldCase {
                field: field.clone(),
                expected: expected.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum UnknownConditionDto {
    Trigger,
    Operator {
        operator: String,
    },
    MalformedArgument,
    StructuredArgument {
        keys: Vec<String>,
    },
    Context {
        reason: ContextReasonDto,
        scope_path: Vec<String>,
        calls: Vec<String>,
        expected: Option<String>,
        actual: Option<String>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ContextReasonDto {
    MissingPreviousScope,
    MissingEventSource,
    MissingFounderSpecies,
    UnknownSpeciesRelation,
    UnknownOwner,
    UnknownTrigger,
    ExpectedScalar,
    UnboundArgument,
    InvalidBoolean,
    MissingIdentity,
    UnverifiedStructure,
    UnknownScope,
    UnknownCollection,
    RecursiveScript,
    TypeMismatch,
}
impl From<&application::UnknownConditionReason> for UnknownConditionDto {
    fn from(value: &application::UnknownConditionReason) -> Self {
        use application::{ContextReason as C, UnknownConditionReason as R};
        match value {
            R::Trigger(_) => Self::Trigger,
            R::Operator { operator, .. } => Self::Operator {
                operator: operator.symbol().into(),
            },
            R::MalformedArgument(_) => Self::MalformedArgument,
            R::StructuredArgument { keys, .. } => Self::StructuredArgument { keys: keys.clone() },
            R::Context {
                reason,
                scope_path,
                calls,
                ..
            } => {
                let (expected, actual) = if let C::TypeMismatch { expected, actual } = reason {
                    (Some(expected.clone()), actual.clone())
                } else {
                    (None, None)
                };
                Self::Context {
                    reason: match reason {
                        C::MissingPreviousScope => ContextReasonDto::MissingPreviousScope,
                        C::MissingEventSource => ContextReasonDto::MissingEventSource,
                        C::MissingFounderSpecies => ContextReasonDto::MissingFounderSpecies,
                        C::UnknownSpeciesRelation => ContextReasonDto::UnknownSpeciesRelation,
                        C::UnknownOwner => ContextReasonDto::UnknownOwner,
                        C::UnknownTrigger => ContextReasonDto::UnknownTrigger,
                        C::ExpectedScalar => ContextReasonDto::ExpectedScalar,
                        C::UnboundArgument => ContextReasonDto::UnboundArgument,
                        C::InvalidBoolean => ContextReasonDto::InvalidBoolean,
                        C::MissingIdentity => ContextReasonDto::MissingIdentity,
                        C::UnverifiedStructure => ContextReasonDto::UnverifiedStructure,
                        C::UnknownScope => ContextReasonDto::UnknownScope,
                        C::UnknownCollection => ContextReasonDto::UnknownCollection,
                        C::RecursiveScript => ContextReasonDto::RecursiveScript,
                        C::TypeMismatch { .. } => ContextReasonDto::TypeMismatch,
                    },
                    scope_path: scope_path.clone(),
                    calls: calls.clone(),
                    expected,
                    actual,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum LocalisationFailureKindDto {
    InvalidUtf8,
    ReadFailed,
}
impl From<application::LocalisationFailureKind> for LocalisationFailureKindDto {
    fn from(value: application::LocalisationFailureKind) -> Self {
        match value {
            application::LocalisationFailureKind::InvalidUtf8 => Self::InvalidUtf8,
            application::LocalisationFailureKind::ReadFailed => Self::ReadFailed,
        }
    }
}
