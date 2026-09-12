use jomini::TextTape;
use jomini::Utf8Encoding;
use jomini::text::ObjectReader;

use crate::error::Error;

pub(crate) type Utf8Object<'data, 'tokens> = ObjectReader<'data, 'tokens, Utf8Encoding>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClausewitzErrorKind {
    InvalidUtf8,
    InvalidSyntax,
}

#[derive(Debug, Clone)]
pub(crate) struct ClausewitzError {
    pub kind: ClausewitzErrorKind,
    pub offset: Option<usize>,
    pub message: String,
}

impl ClausewitzError {
    pub(crate) fn invalid_utf8(offset: usize) -> Self {
        Self {
            kind: ClausewitzErrorKind::InvalidUtf8,
            offset: Some(offset),
            message: format!("invalid UTF-8 at byte {offset}"),
        }
    }

    pub(crate) fn from_jomini(error: jomini::Error) -> Self {
        Self {
            kind: ClausewitzErrorKind::InvalidSyntax,
            offset: error.offset(),
            message: error.to_string(),
        }
    }
}

impl From<ClausewitzError> for Error {
    fn from(error: ClausewitzError) -> Self {
        Error::Parse {
            message: error.message.clone(),
            offset: error.offset,
        }
    }
}

impl ClausewitzError {
    pub(crate) fn display_message(&self) -> String {
        match self.offset {
            Some(offset) => format!("{} (byte offset {offset})", self.message),
            None => self.message.clone(),
        }
    }
}

pub(crate) struct ClausewitzDocument<'data> {
    tape: TextTape<'data>,
}

impl<'data> ClausewitzDocument<'data> {
    pub(crate) fn parse(data: &'data [u8]) -> Result<Self, ClausewitzError> {
        if let Err(error) = std::str::from_utf8(data) {
            return Err(ClausewitzError::invalid_utf8(error.valid_up_to()));
        }
        let tape = TextTape::from_slice(data).map_err(ClausewitzError::from_jomini)?;
        Ok(Self { tape })
    }

    pub(crate) fn root(&self) -> Utf8Object<'data, '_> {
        self.tape.utf8_reader()
    }
}
