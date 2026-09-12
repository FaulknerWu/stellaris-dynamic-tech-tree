use serde::{Deserialize, Serialize};

use crate::clausewitz::ClausewitzDocument;
use crate::error::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub name: Option<String>,
    pub date: Option<String>,
    pub version: Option<String>,
}

impl SaveMetadata {
    pub(super) fn parse(bytes: &[u8]) -> Result<Self> {
        let document = ClausewitzDocument::parse(bytes)?;
        let root = document.root();
        let mut metadata = Self::default();

        for (key, _operator, value) in root.fields() {
            match key.read_string().as_str() {
                "name" if metadata.name.is_none() => metadata.name = value.read_string().ok(),
                "date" if metadata.date.is_none() => metadata.date = value.read_string().ok(),
                "version" if metadata.version.is_none() => {
                    metadata.version = value.read_string().ok()
                }
                _ => {}
            }
        }

        Ok(metadata)
    }
}
