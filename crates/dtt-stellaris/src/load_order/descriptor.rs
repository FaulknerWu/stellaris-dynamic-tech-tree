use std::fs;
use std::path::Path;

use crate::clausewitz::{ClausewitzDocument, read_scalar};
use crate::error::{Error, Result};

#[derive(Debug, Default, Clone)]
struct Descriptor {
    replace_paths: Vec<String>,
}

pub(super) fn read_replace_paths(path: &Path) -> Result<Vec<String>> {
    let bytes = fs::read(path).map_err(|error| Error::io(path, error))?;
    Ok(parse_descriptor(&bytes)?.replace_paths)
}

fn parse_descriptor(src: &[u8]) -> Result<Descriptor> {
    let document = ClausewitzDocument::parse(src)
        .map_err(|error| Error::LoadOrder(error.display_message()))?;
    let root = document.root();
    let mut replace_paths = Vec::new();
    for (key, _operator, value) in root.fields() {
        if key.read_str() != "replace_path" {
            continue;
        }
        let path = read_scalar(&value)
            .map_err(|_| Error::LoadOrder("`replace_path` value is not a scalar".into()))?;
        replace_paths.push(path);
    }
    Ok(Descriptor { replace_paths })
}
