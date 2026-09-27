use crate::{TranslationError, Translator};

#[derive(Debug, Clone, Copy)]
pub enum ReportLabel {
    Title,
    MissingMods,
    Eligible,
    Uncertain,
    ExcludedIdentity,
    ExcludedPrerequisite,
    Unknown,
    Deferred,
    GameData,
    Definitions,
    Localisation,
    SelfCycles,
    Cycles,
    SwapMatched,
    SwapNoMatch,
    SwapUncertain,
    Technical,
}
impl Translator {
    pub fn report_label(&self, label: ReportLabel) -> Result<String, TranslationError> {
        self.text(
            match label {
                ReportLabel::Title => "report-title",
                ReportLabel::MissingMods => "report-missing-mods",
                ReportLabel::Eligible => "report-eligible",
                ReportLabel::Uncertain => "report-uncertain",
                ReportLabel::ExcludedIdentity => "report-excluded-identity",
                ReportLabel::ExcludedPrerequisite => "report-excluded-prerequisite",
                ReportLabel::Unknown => "report-unknown",
                ReportLabel::Deferred => "report-deferred",
                ReportLabel::GameData => "report-game-data",
                ReportLabel::Definitions => "report-definitions",
                ReportLabel::Localisation => "report-localisation",
                ReportLabel::SelfCycles => "report-self-cycles",
                ReportLabel::Cycles => "report-cycles",
                ReportLabel::SwapMatched => "report-swap-matched",
                ReportLabel::SwapNoMatch => "report-swap-no-match",
                ReportLabel::SwapUncertain => "report-swap-uncertain",
                ReportLabel::Technical => "report-technical",
            },
            None,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CliMessage {
    About,
    Generate,
    DetectPaths,
    Usage,
    Commands,
    Options,
    Arguments,
    Help,
    Version,
    Locale,
    SaveFile,
    GameRoot,
    Documents,
    Launcher,
    Language,
    UnknownStrategy,
    SwapStrategy,
    NotDetected,
    Libraries,
    Output,
    Success,
    Incomplete,
    Written,
    Removed,
    Failed,
    InvalidLocale,
    InvalidEnvironmentLocale,
    ParseError,
    UnknownArgument,
    InvalidValue,
    MissingArgument,
    Conflict,
    Technical,
}
impl Translator {
    pub fn cli(&self, label: CliMessage) -> Result<String, TranslationError> {
        self.text(
            match label {
                CliMessage::About => "cli-about",
                CliMessage::Generate => "cli-generate",
                CliMessage::DetectPaths => "cli-detect-paths",
                CliMessage::Usage => "cli-usage",
                CliMessage::Commands => "cli-commands",
                CliMessage::Options => "cli-options",
                CliMessage::Arguments => "cli-arguments",
                CliMessage::Help => "cli-help",
                CliMessage::Version => "cli-version",
                CliMessage::Locale => "cli-locale",
                CliMessage::SaveFile => "cli-save-file",
                CliMessage::GameRoot => "cli-game-root",
                CliMessage::Documents => "cli-documents",
                CliMessage::Launcher => "cli-launcher",
                CliMessage::Language => "cli-language",
                CliMessage::UnknownStrategy => "cli-unknown-strategy",
                CliMessage::SwapStrategy => "cli-swap-strategy",
                CliMessage::NotDetected => "cli-not-detected",
                CliMessage::Libraries => "cli-libraries",
                CliMessage::Output => "cli-output",
                CliMessage::Success => "cli-success",
                CliMessage::Incomplete => "cli-incomplete",
                CliMessage::Written => "cli-written",
                CliMessage::Removed => "cli-removed",
                CliMessage::Failed => "cli-failed",
                CliMessage::InvalidLocale => "cli-invalid-locale",
                CliMessage::InvalidEnvironmentLocale => "cli-invalid-environment-locale",
                CliMessage::ParseError => "cli-parse-error",
                CliMessage::UnknownArgument => "cli-unknown-argument",
                CliMessage::InvalidValue => "cli-invalid-value",
                CliMessage::MissingArgument => "cli-missing-argument",
                CliMessage::Conflict => "cli-conflict",
                CliMessage::Technical => "cli-technical",
            },
            None,
        )
    }
}

impl Translator {
    pub fn diagnostic_invalid_utf8(&self) -> Result<String, TranslationError> {
        self.text("diagnostic-invalid-utf8", None)
    }
    pub fn diagnostic_invalid_syntax(&self) -> Result<String, TranslationError> {
        self.text("diagnostic-invalid-syntax", None)
    }
    pub fn diagnostic_expected_scalar(&self) -> Result<String, TranslationError> {
        self.text("diagnostic-expected-scalar", None)
    }
    pub fn diagnostic_expected_object(&self) -> Result<String, TranslationError> {
        self.text("diagnostic-expected-object", None)
    }
    pub fn diagnostic_invalid_field(&self, field: &str) -> Result<String, TranslationError> {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("field", field);
        self.text("diagnostic-invalid-field", Some(&args))
    }
    pub fn diagnostic_overwritten(&self, previous: &str) -> Result<String, TranslationError> {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("previous", previous);
        self.text("diagnostic-overwritten", Some(&args))
    }
    pub fn diagnostic_inline_cycle(&self, script: &str) -> Result<String, TranslationError> {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("script", script);
        self.text("diagnostic-inline-cycle", Some(&args))
    }
    pub fn diagnostic_inline_missing(&self, script: &str) -> Result<String, TranslationError> {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("script", script);
        self.text("diagnostic-inline-missing", Some(&args))
    }
    pub fn diagnostic_invalid_inline_call(&self) -> Result<String, TranslationError> {
        self.text("diagnostic-invalid-inline-call", None)
    }
    pub fn diagnostic_isolated_branch(&self) -> Result<String, TranslationError> {
        self.text("diagnostic-isolated-branch", None)
    }
    pub fn diagnostic_invalid_condition(&self) -> Result<String, TranslationError> {
        self.text("diagnostic-invalid-condition", None)
    }
    pub fn diagnostic_malformed_argument(&self) -> Result<String, TranslationError> {
        self.text("diagnostic-malformed-argument", None)
    }
    pub fn diagnostic_structured_argument(&self) -> Result<String, TranslationError> {
        self.text("diagnostic-structured-argument", None)
    }
    pub fn diagnostic_duplicate_parameter(&self) -> Result<String, TranslationError> {
        self.text("diagnostic-duplicate-parameter", None)
    }
    pub fn diagnostic_invalid_swap(
        &self,
        index: usize,
        field: &str,
    ) -> Result<String, TranslationError> {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("index", index);
        args.set("field", field);
        self.text("diagnostic-invalid-swap", Some(&args))
    }
    pub fn diagnostic_duplicate_field(&self, field: &str) -> Result<String, TranslationError> {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("field", field);
        self.text("diagnostic-duplicate-field", Some(&args))
    }
    pub fn diagnostic_field_case(
        &self,
        field: &str,
        expected: &str,
    ) -> Result<String, TranslationError> {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("field", field);
        args.set("expected", expected);
        self.text("diagnostic-field-case", Some(&args))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorMessage {
    MissingSetting,
    InvalidPath,
    NoOutputLanguage,
    ExecutablePathUnavailable,
    NoTechnologyDefinitions,

    SaveUnavailable,
    SaveContainerCorrupt,
    UnsupportedBinarySave,
    PlayerCountryMissing,
    PlayerCountryRequired,
    LauncherDatabaseUnavailable,
    GenerationBusy,
    GenerationCancelled,
    Internal,
}
impl Translator {
    pub fn error(&self, code: ErrorMessage) -> Result<String, TranslationError> {
        self.text(
            match code {
                ErrorMessage::SaveUnavailable => "error-save-unavailable",
                ErrorMessage::SaveContainerCorrupt => "error-save-container-corrupt",
                ErrorMessage::UnsupportedBinarySave => "error-unsupported-binary-save",
                ErrorMessage::PlayerCountryMissing => "error-player-country-missing",
                ErrorMessage::PlayerCountryRequired => "error-player-country-required",
                ErrorMessage::LauncherDatabaseUnavailable => "error-launcher-database-unavailable",
                ErrorMessage::GenerationBusy => "error-generation-busy",
                ErrorMessage::GenerationCancelled => "error-generation-cancelled",
                ErrorMessage::Internal => "error-internal",
                ErrorMessage::MissingSetting => "error-missing-setting",
                ErrorMessage::InvalidPath => "error-invalid-path",
                ErrorMessage::NoOutputLanguage => "error-no-output-language",
                ErrorMessage::ExecutablePathUnavailable => "error-executable-path-unavailable",
                ErrorMessage::NoTechnologyDefinitions => "error-no-technology-definitions",
            },
            None,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StageMessage {
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
impl Translator {
    pub fn stage(&self, stage: StageMessage) -> Result<String, TranslationError> {
        self.text(
            match stage {
                StageMessage::SaveParse => "stage-save-parse",
                StageMessage::LoadOrder => "stage-load-order",
                StageMessage::IngestTech => "stage-ingest-tech",
                StageMessage::Relations => "stage-relations",
                StageMessage::IngestL10n => "stage-ingest-l10n",
                StageMessage::Render => "stage-render",
                StageMessage::Cycles => "stage-cycles",
                StageMessage::WriteOutput => "stage-write-output",
                StageMessage::Done => "stage-done",
            },
            None,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConditionMessage {
    Trigger,
    Operator,
    MalformedArgument,
    StructuredArgument,
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
impl Translator {
    pub fn condition(&self, reason: ConditionMessage) -> Result<String, TranslationError> {
        self.text(
            match reason {
                ConditionMessage::Trigger => "condition-trigger",
                ConditionMessage::Operator => "condition-operator",
                ConditionMessage::MalformedArgument => "condition-malformed-argument",
                ConditionMessage::StructuredArgument => "condition-structured-argument",
                ConditionMessage::MissingPreviousScope => "condition-missing-previous-scope",
                ConditionMessage::MissingEventSource => "condition-missing-event-source",
                ConditionMessage::MissingFounderSpecies => "condition-missing-founder-species",
                ConditionMessage::UnknownSpeciesRelation => "condition-unknown-species-relation",
                ConditionMessage::UnknownOwner => "condition-unknown-owner",
                ConditionMessage::UnknownTrigger => "condition-unknown-trigger",
                ConditionMessage::ExpectedScalar => "condition-expected-scalar",
                ConditionMessage::UnboundArgument => "condition-unbound-argument",
                ConditionMessage::InvalidBoolean => "condition-invalid-boolean",
                ConditionMessage::MissingIdentity => "condition-missing-identity",
                ConditionMessage::UnverifiedStructure => "condition-unverified-structure",
                ConditionMessage::UnknownScope => "condition-unknown-scope",
                ConditionMessage::UnknownCollection => "condition-unknown-collection",
                ConditionMessage::RecursiveScript => "condition-recursive-script",
                ConditionMessage::TypeMismatch => "condition-type-mismatch",
            },
            None,
        )
    }
}

impl Translator {
    pub fn cli_summary(
        &self,
        definitions: usize,
        eligible: usize,
        sources: usize,
    ) -> Result<String, TranslationError> {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("definitions", definitions);
        args.set("eligible", eligible);
        args.set("sources", sources);
        self.text("cli-summary", Some(&args))
    }
}

impl Translator {
    pub fn localisation_failure(&self, invalid_utf8: bool) -> Result<String, TranslationError> {
        self.text(
            if invalid_utf8 {
                "diagnostic-invalid-utf8"
            } else {
                "diagnostic-read-failed"
            },
            None,
        )
    }
}
