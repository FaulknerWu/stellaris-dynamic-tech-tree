use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension};

use super::ModRef;
use super::descriptor;
use crate::error::{Error, Result};

const SQL_PLAYSET_MODS: &str = "\
SELECT m.dirPath, m.displayName \
FROM playsets_mods pm \
JOIN mods m ON m.id = pm.modId \
WHERE pm.playsetId = ?1 AND pm.enabled = 1 \
ORDER BY pm.position ASC, m.displayName ASC";

pub(super) fn read(path: &Path) -> Result<Vec<ModRef>> {
    let conn = Connection::open(path).map_err(|error| launcher_err(path, error))?;
    let playset = active_playset(&conn, path)?;
    let Some(playset_id) = playset else {
        return Ok(Vec::new());
    };

    let mut stmt = conn
        .prepare(SQL_PLAYSET_MODS)
        .map_err(|error| launcher_err(path, error))?;
    let rows = stmt
        .query_map([&playset_id], |row| {
            Ok(ModRow {
                dir_path: row.get::<_, Option<String>>(0)?,
                display_name: row.get::<_, Option<String>>(1)?,
            })
        })
        .map_err(|error| launcher_err(path, error))?;

    let mut out = Vec::new();
    for row in rows {
        let row = row.map_err(|error| launcher_err(path, error))?;
        let (root, descriptor_path) = resolve_db_mod_root(row.dir_path.as_deref())?;
        let replace_paths = descriptor::read_replace_paths(&descriptor_path)?;
        let name = required_display_name(row.display_name.as_deref())?;
        out.push(ModRef {
            name,
            root,
            replace_paths,
        });
    }
    Ok(out)
}

fn active_playset(conn: &Connection, path: &Path) -> Result<Option<String>> {
    conn.query_row(
        "SELECT id FROM playsets WHERE isActive = 1 ORDER BY name LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(|error| launcher_err(path, error))
}

struct ModRow {
    dir_path: Option<String>,
    display_name: Option<String>,
}

fn resolve_db_mod_root(dir_path: Option<&str>) -> Result<(PathBuf, PathBuf)> {
    if let Some(dir_path) = dir_path.filter(|value| !value.trim().is_empty()) {
        let root = PathBuf::from(dir_path);
        return Ok((root.clone(), root.join("descriptor.mod")));
    }
    Err(Error::LoadOrder(
        "Mod in launcher database is missing dirPath".into(),
    ))
}

fn required_display_name(display_name: Option<&str>) -> Result<String> {
    display_name
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| Error::LoadOrder("Mod in launcher database is missing displayName".into()))
}

fn launcher_err(path: &Path, error: rusqlite::Error) -> Error {
    Error::LauncherDb {
        path: path.to_path_buf(),
        source: error,
    }
}
