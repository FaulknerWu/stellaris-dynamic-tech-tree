use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::path::Path;

use super::{Archive, Gamestate, GamestateKind, SaveContainerIndex, SaveMetadata};
use crate::error::{Error, Result};

pub fn open(path: &Path) -> Result<Archive> {
    let file = File::open(path).map_err(|error| Error::io(path, error))?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|error| Error::Container(error.to_string()))?;

    let meta_bytes = read_entry(&mut zip, "meta")?;
    let metadata = SaveMetadata::parse(&meta_bytes)?;

    let gamestate = read_entry(&mut zip, "gamestate")?;
    let gamestate = if looks_binary(&gamestate) {
        Gamestate::Binary
    } else {
        Gamestate::Text(gamestate)
    };

    Ok(Archive {
        metadata,
        gamestate,
    })
}

pub(crate) fn index(path: &Path) -> Result<SaveContainerIndex> {
    let file = File::open(path).map_err(|error| Error::io(path, error))?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|error| Error::Container(error.to_string()))?;
    let metadata = SaveMetadata::parse(&read_entry(&mut zip, "meta")?)?;
    let gamestate_head = read_entry_prefix(&mut zip, "gamestate", 128)?;
    let gamestate_kind = if looks_binary(&gamestate_head) {
        GamestateKind::Binary
    } else {
        GamestateKind::Text
    };
    Ok(SaveContainerIndex {
        metadata,
        gamestate_kind,
    })
}

fn looks_binary(head: &[u8]) -> bool {
    let window = &head[..head.len().min(128)];
    window.contains(&0) || window.starts_with(b"bin") || window.starts_with(b"sav")
}

fn read_entry<R: Read + Seek>(zip: &mut zip::ZipArchive<R>, name: &str) -> Result<Vec<u8>> {
    let mut entry = zip
        .by_name(name)
        .map_err(|error| Error::Container(format!("missing `{name}` entry: {error}")))?;
    let mut buf = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut buf)
        .map_err(|error| Error::Container(format!("failed to read `{name}`: {error}")))?;
    Ok(buf)
}

fn read_entry_prefix<R: Read + Seek>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
    limit: u64,
) -> Result<Vec<u8>> {
    let mut entry = zip
        .by_name(name)
        .map_err(|error| Error::Container(format!("missing `{name}` entry: {error}")))?;
    let mut buf = Vec::with_capacity(limit as usize);
    entry
        .by_ref()
        .take(limit)
        .read_to_end(&mut buf)
        .map_err(|error| Error::Container(format!("failed to read `{name}` prefix: {error}")))?;
    Ok(buf)
}
