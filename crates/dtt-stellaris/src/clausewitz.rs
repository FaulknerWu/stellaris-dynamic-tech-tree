mod document;
mod reader;
pub(crate) mod script;
pub(crate) mod value;

pub(crate) use document::{ClausewitzDocument, ClausewitzError, ClausewitzErrorKind, Utf8Object};
pub(crate) use reader::{read_object, read_scalar, read_scalar_values};
