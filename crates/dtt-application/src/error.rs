pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Translation(#[from] dtt_i18n::TranslationError),
    #[error("settings error: {0}")]
    Settings(SettingsIssue),
    #[error("generation cancelled")]
    Cancelled,
    #[error("save file does not exist or is not accessible: {}", .0.display())]
    SaveUnavailable(std::path::PathBuf),
    #[error("binary or ironman saves are not supported")]
    UnsupportedBinarySave,
    #[error("no player country was found in the save")]
    PlayerCountryMissing,
    #[error("multiple player countries were found; one must be selected")]
    PlayerCountryRequired,
    #[error(transparent)]
    Domain(#[from] dtt_core::Error),
    #[error(transparent)]
    Stellaris(#[from] dtt_stellaris::Error),
    #[error("current_exe: {0}")]
    ExecutablePath(#[source] std::io::Error),
    #[error("executable_parent_missing")]
    ExecutableParentMissing,
    #[error("no_technology_definitions")]
    NoTechnologyDefinitions,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{self:?}")]
pub enum SettingsIssue {
    Missing {
        field: &'static str,
    },
    InvalidPath {
        field: &'static str,
        path: std::path::PathBuf,
    },
    NoOutputLanguage,
}
