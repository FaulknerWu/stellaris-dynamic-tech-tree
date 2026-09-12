use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to access `{}`", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("clausewitz parse error: {message}{}", format_offset(*.offset))]
    Parse {
        message: String,
        offset: Option<usize>,
    },
    #[error("save container error: {0}")]
    Container(String),
    #[error("load-order error: {0}")]
    LoadOrder(String),
    #[error("launcher database error at `{}`", path.display())]
    LauncherDb {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
    #[error("save snapshot error: {0}")]
    Snapshot(String),
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

fn format_offset(offset: Option<usize>) -> String {
    match offset {
        Some(offset) => format!(" (byte offset {offset})"),
        None => String::new(),
    }
}

impl From<jomini::DeserializeError> for Error {
    fn from(error: jomini::DeserializeError) -> Self {
        Self::Parse {
            message: error.to_string(),
            offset: None,
        }
    }
}
