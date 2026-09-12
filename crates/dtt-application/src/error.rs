pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("settings error: {0}")]
    Settings(String),
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
    #[error("application error: {0}")]
    Application(String),
}
