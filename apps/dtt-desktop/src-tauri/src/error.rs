use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::dto::ErrorContextDto;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidEnvironment,
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
    pub message: String,
    #[ts(optional)]
    pub detail: Option<String>,
    #[ts(optional)]
    pub context: Option<ErrorContextDto>,
}

impl AppError {
    pub fn generation_busy() -> Self {
        Self::new(ErrorCode::GenerationBusy, "已有生成任务正在运行", None)
    }

    pub fn player_country_required() -> Self {
        Self::new(
            ErrorCode::PlayerCountryRequired,
            "该存档包含多个玩家国家，请先选择一个国家",
            None,
        )
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, "发生内部错误", Some(detail.into()))
    }

    fn new(code: ErrorCode, message: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            code,
            message: message.into(),
            detail,
            context: None,
        }
    }
}

impl From<dtt_application::Error> for AppError {
    fn from(error: dtt_application::Error) -> Self {
        use dtt_application::Error;

        let detail = error.to_string();
        match error {
            Error::Settings(_) => {
                Self::new(ErrorCode::InvalidEnvironment, "环境配置无效", Some(detail))
            }
            Error::Cancelled => Self::new(ErrorCode::GenerationCancelled, "生成任务已取消", None),
            Error::SaveUnavailable(path) => Self {
                code: ErrorCode::SaveUnavailable,
                message: "存档不存在或不可访问".into(),
                detail: Some(detail),
                context: Some(ErrorContextDto {
                    path: Some(path.to_string_lossy().into_owned()),
                }),
            },
            Error::UnsupportedBinarySave => Self::new(
                ErrorCode::UnsupportedBinarySave,
                "不支持二进制或铁人存档",
                None,
            ),
            Error::PlayerCountryMissing => {
                Self::new(ErrorCode::PlayerCountryMissing, "存档中没有玩家国家", None)
            }
            Error::PlayerCountryRequired => Self::player_country_required(),
            Error::Stellaris(dtt_application::StellarisError::Container(_)) => Self::new(
                ErrorCode::SaveContainerCorrupt,
                "存档容器损坏",
                Some(detail),
            ),
            Error::Stellaris(dtt_application::StellarisError::LauncherDb { .. }) => Self::new(
                ErrorCode::LauncherDatabaseUnavailable,
                "启动器数据库不可访问",
                Some(detail),
            ),
            _ => Self::internal(detail),
        }
    }
}
