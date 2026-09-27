use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::dto::ErrorContextDto;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: ErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub technical_detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub context: Option<ErrorContextDto>,
}

impl AppError {
    pub fn generation_busy() -> Self {
        Self::new(ErrorCode::GenerationBusy, None)
    }

    pub fn player_country_required() -> Self {
        Self::new(ErrorCode::PlayerCountryRequired, None)
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, Some(detail.into()))
    }

    fn new(code: ErrorCode, detail: Option<String>) -> Self {
        Self {
            code,
            technical_detail: detail,
            context: None,
        }
    }
}

impl From<dtt_application::Error> for AppError {
    fn from(error: dtt_application::Error) -> Self {
        use dtt_application::Error;

        let detail = error.to_string();
        match error {
            Error::Settings(issue) => {
                use dtt_application::SettingsIssue;
                let (code, path) = match issue {
                    SettingsIssue::Missing { .. } => (ErrorCode::MissingSetting, None),
                    SettingsIssue::InvalidPath { path, .. } => (
                        ErrorCode::InvalidPath,
                        Some(path.to_string_lossy().into_owned()),
                    ),
                    SettingsIssue::NoOutputLanguage => (ErrorCode::NoOutputLanguage, None),
                };
                Self {
                    code,
                    technical_detail: Some(detail),
                    context: path.map(|path| ErrorContextDto { path: Some(path) }),
                }
            }
            Error::NoTechnologyDefinitions => Self::new(ErrorCode::NoTechnologyDefinitions, None),
            Error::ExecutablePath(_) | Error::ExecutableParentMissing => {
                Self::new(ErrorCode::ExecutablePathUnavailable, Some(detail))
            }
            Error::Cancelled => Self::new(ErrorCode::GenerationCancelled, None),
            Error::SaveUnavailable(path) => Self {
                code: ErrorCode::SaveUnavailable,
                technical_detail: Some(detail),
                context: Some(ErrorContextDto {
                    path: Some(path.to_string_lossy().into_owned()),
                }),
            },
            Error::UnsupportedBinarySave => Self::new(ErrorCode::UnsupportedBinarySave, None),
            Error::PlayerCountryMissing => Self::new(ErrorCode::PlayerCountryMissing, None),
            Error::PlayerCountryRequired => Self::player_country_required(),
            Error::Stellaris(dtt_application::StellarisError::Container(_)) => {
                Self::new(ErrorCode::SaveContainerCorrupt, Some(detail))
            }
            Error::Stellaris(dtt_application::StellarisError::LauncherDb { .. }) => {
                Self::new(ErrorCode::LauncherDatabaseUnavailable, Some(detail))
            }
            _ => Self::internal(detail),
        }
    }
}
