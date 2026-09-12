use std::collections::HashSet;

use dtt_core::condition::GraphicalCultures;

use crate::clausewitz::{ClausewitzDocument, read_object, read_scalar_values};
use crate::error::Result;
use crate::load_order::Manifest;

use super::diagnostic::{
    GameDataCategory, Ingested, definition_diagnostic, overwrite_diagnostic, parse_diagnostic,
};
use super::{manifest_txt_files, read_bytes};

const GRAPHICAL_CULTURE_DIR: &str = "common/graphical_culture";

pub(crate) fn ingest_graphical_cultures(
    manifest: &Manifest,
) -> Result<Ingested<GraphicalCultures>> {
    let mut cultures = GraphicalCultures::new();
    let mut diagnostics = Vec::new();
    for file in manifest_txt_files(manifest, GRAPHICAL_CULTURE_DIR)? {
        let bytes = read_bytes(&file.path)?;
        match ClausewitzDocument::parse(&bytes) {
            Ok(document) => collect_cultures(
                document.root(),
                &file.provenance(),
                &mut cultures,
                &mut diagnostics,
            ),
            Err(error) => diagnostics.push(parse_diagnostic(
                file.provenance(),
                GameDataCategory::GraphicalCulture,
                &error,
            )),
        }
    }
    Ok(Ingested::new(cultures, diagnostics))
}

fn collect_cultures(
    root: crate::clausewitz::Utf8Object<'_, '_>,
    source: &str,
    cultures: &mut GraphicalCultures,
    diagnostics: &mut Vec<super::diagnostic::GameDataDiagnostic>,
) {
    for (key, _operator, value) in root.fields() {
        let name = key.read_string();
        if name.starts_with('@') {
            continue;
        }
        let Ok(object) = read_object(&value) else {
            continue;
        };
        let Some(ship_kinds) = object
            .fields()
            .find_map(|(key, _operator, value)| (key.read_str() == "ship_kinds").then_some(value))
        else {
            continue;
        };
        match read_scalar_values(&ship_kinds) {
            Ok(kinds) => {
                if cultures
                    .insert(name.clone(), kinds.into_iter().collect::<HashSet<_>>())
                    .is_some()
                {
                    diagnostics.push(overwrite_diagnostic(
                        source,
                        GameDataCategory::GraphicalCulture,
                        &name,
                        format!("图形文化 `{name}` 被 `{source}` 覆盖"),
                    ));
                }
            }
            Err(_) => diagnostics.push(definition_diagnostic(
                source,
                GameDataCategory::GraphicalCulture,
                &name,
                format!("图形文化 `{name}` 的 `ship_kinds` 不是可读的标量数组"),
            )),
        }
    }
}
