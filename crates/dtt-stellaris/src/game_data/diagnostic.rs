use serde::Serialize;

use crate::clausewitz::{ClausewitzError, ClausewitzErrorKind};

#[derive(Debug, Clone)]
pub struct Ingested<T> {
    pub value: T,
    pub diagnostics: Vec<GameDataDiagnostic>,
}

impl<T> Ingested<T> {
    pub(crate) fn new(value: T, mut diagnostics: Vec<GameDataDiagnostic>) -> Self {
        sort_diagnostics(&mut diagnostics);
        Self { value, diagnostics }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GameDataDiagnostic {
    pub source: String,
    pub category: GameDataCategory,
    pub subject: Option<String>,
    pub byte_offset: Option<usize>,
    pub kind: GameDataDiagnosticKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameDataCategory {
    ScriptedVariable,
    Technology,
    ScriptedTrigger,
    GraphicalCulture,
    InlineScript,
}

impl GameDataCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ScriptedVariable => "scripted_variable",
            Self::Technology => "technology",
            Self::ScriptedTrigger => "scripted_trigger",
            Self::GraphicalCulture => "graphical_culture",
            Self::InlineScript => "inline_script",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameDataDiagnosticKind {
    InvalidUtf8,
    InvalidSyntax,
    InvalidDefinition,
    UnsupportedCondition,
    DuplicateParameter,
    OverwrittenDefinition,
}

impl GameDataDiagnosticKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidUtf8 => "invalid_utf8",
            Self::InvalidSyntax => "invalid_syntax",
            Self::InvalidDefinition => "invalid_definition",
            Self::UnsupportedCondition => "unsupported_condition",
            Self::DuplicateParameter => "duplicate_parameter",
            Self::OverwrittenDefinition => "overwritten_definition",
        }
    }
}

pub(crate) fn sort_diagnostics(diagnostics: &mut [GameDataDiagnostic]) {
    diagnostics.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then(left.category.cmp(&right.category))
            .then(left.subject.cmp(&right.subject))
            .then(left.byte_offset.cmp(&right.byte_offset))
            .then(left.kind.cmp(&right.kind))
            .then(left.message.cmp(&right.message))
    });
}

pub(crate) fn parse_diagnostic(
    source: impl Into<String>,
    category: GameDataCategory,
    error: &ClausewitzError,
) -> GameDataDiagnostic {
    let kind = match error.kind {
        ClausewitzErrorKind::InvalidUtf8 => GameDataDiagnosticKind::InvalidUtf8,
        ClausewitzErrorKind::InvalidSyntax => GameDataDiagnosticKind::InvalidSyntax,
    };
    let message = match error.kind {
        ClausewitzErrorKind::InvalidUtf8 => match error.offset {
            Some(offset) => format!("文件不是有效的 UTF-8（字节偏移 {offset}）"),
            None => "文件不是有效的 UTF-8".to_string(),
        },
        ClausewitzErrorKind::InvalidSyntax => match error.offset {
            Some(offset) => format!(
                "Clausewitz 语法错误（字节偏移 {offset}）：{}",
                error.message
            ),
            None => format!("Clausewitz 语法错误：{}", error.message),
        },
    };
    GameDataDiagnostic {
        source: source.into(),
        category,
        subject: None,
        byte_offset: error.offset,
        kind,
        message,
    }
}

pub(crate) fn definition_diagnostic(
    source: impl Into<String>,
    category: GameDataCategory,
    subject: impl Into<String>,
    message: impl Into<String>,
) -> GameDataDiagnostic {
    GameDataDiagnostic {
        source: source.into(),
        category,
        subject: Some(subject.into()),
        byte_offset: None,
        kind: GameDataDiagnosticKind::InvalidDefinition,
        message: message.into(),
    }
}

pub(crate) fn overwrite_diagnostic(
    source: impl Into<String>,
    category: GameDataCategory,
    subject: impl Into<String>,
    message: impl Into<String>,
) -> GameDataDiagnostic {
    GameDataDiagnostic {
        source: source.into(),
        category,
        subject: Some(subject.into()),
        byte_offset: None,
        kind: GameDataDiagnosticKind::OverwrittenDefinition,
        message: message.into(),
    }
}
