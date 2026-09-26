use crate::clausewitz::ClausewitzDocument;
use crate::error::{Error, Result};
use dtt_core::empire::{Government, Snapshot, Species};
use jomini::text::{ObjectReader, ValueReader};
use serde::Serialize;

type Obj<'d, 't> = ObjectReader<'d, 't, jomini::Utf8Encoding>;

type Val<'d, 't> = ValueReader<'d, 't, jomini::Utf8Encoding>;

#[derive(Debug, Clone, Serialize)]
pub struct PlayerCountryCandidate {
    pub country_id: i64,
}

pub fn player_country_candidates(gamestate: &[u8]) -> Result<Vec<PlayerCountryCandidate>> {
    let document = ClausewitzDocument::parse_save(gamestate)?;
    let root = document.root();
    Ok(collect_player_country_ids(&root)?
        .into_iter()
        .map(|country_id| PlayerCountryCandidate { country_id })
        .collect())
}

pub fn extract_snapshot_for_country(gamestate: &[u8], country_id: i64) -> Result<Snapshot> {
    let document = ClausewitzDocument::parse_save(gamestate)?;
    let root = document.root();
    let country = block_by_id(&root, "country", country_id)?;
    let mut snapshot = build_snapshot(&root, country)?;
    if collect_player_country_ids(&root)?.contains(&country_id) {
        snapshot.is_ai = Some(false);
    }
    Ok(snapshot)
}

fn collect_player_country_ids<'d, 't>(root: &Obj<'d, 't>) -> Result<Vec<i64>> {
    let mut out = Vec::new();
    for (key, _op, val) in root.fields() {
        if key.read_string() != "player" {
            continue;
        }
        if let Ok(arr) = val.read_array() {
            for entry in arr.values() {
                if let Ok(player) = entry.read_object() {
                    grab_country_id(&player, &mut out);
                }
            }
        }
    }
    Ok(out)
}

fn grab_country_id<'d, 't>(player: &Obj<'d, 't>, out: &mut Vec<i64>) {
    for (key, _op, val) in player.fields() {
        if key.read_string() == "country"
            && let Some(id) = val.read_scalar().ok().and_then(|s| s.to_i64().ok())
            && !out.contains(&id)
        {
            out.push(id);
        }
    }
}

fn block_by_id<'d, 't>(root: &Obj<'d, 't>, container: &str, id: i64) -> Result<Obj<'d, 't>> {
    for (key, _op, val) in root.fields() {
        if key.read_string() != container {
            continue;
        }
        let inner = val.read_object()?;
        for (id_key, _op, id_val) in inner.fields() {
            let matches = id_key
                .read_scalar()
                .to_i64()
                .ok()
                .is_some_and(|parsed| parsed == id);
            if matches {
                return Ok(id_val.read_object()?);
            }
        }
    }
    Err(Error::Snapshot(format!(
        "No sub-block with id={id} found in `{container}` block"
    )))
}

fn build_snapshot<'d, 't>(root: &Obj<'d, 't>, country: Obj<'d, 't>) -> Result<Snapshot> {
    let mut snap = Snapshot::default();

    let mut founder_ref: Option<i64> = None;

    for (key, _op, val) in country.fields() {
        match key.read_string().as_str() {
            "founder_species_ref" => {
                founder_ref = val.read_scalar().ok().and_then(|s| s.to_i64().ok());
            }

            "ethos" => {
                snap.ethics = Some(collect_repeated_key_values(&val.read_object()?, "ethic"))
            }
            "ascension_perks" => {
                snap.ascension_perks = collect_string_array(&val).unwrap_or_default()
            }
            "traditions" => snap.traditions = collect_string_array(&val).unwrap_or_default(),
            "graphical_culture" => snap.graphical_culture = val.read_string().ok(),
            "type" => snap.country_type = Some(normalise_country_type(&val.read_string()?)),
            "is_nomadic" => snap.is_nomadic = read_bool(&val),
            "is_ai" => snap.is_ai = read_bool(&val),
            "government" => read_government(&mut snap.government, &val.read_object()?),
            _ => {}
        }
    }

    if let Some(ref_id) = founder_ref
        && let Ok(species) = block_by_id(root, "species_db", ref_id)
    {
        let traits = species_traits(&species);
        snap.founder_species = Some(Species {
            archetype: traits
                .as_deref()
                .map(|traits| infer_archetype(traits).to_string()),
            traits,
        });
    }

    Ok(snap)
}

fn read_government<'d, 't>(gov: &mut Government, government: &Obj<'d, 't>) {
    for (key, _op, val) in government.fields() {
        match key.read_string().as_str() {
            "authority" => gov.authority = val.read_string().ok(),
            "origin" => gov.origin = val.read_string().ok(),
            "type" => gov.government_type = val.read_string().ok(),
            "civics" => gov.civics = collect_string_array(&val),
            _ => {}
        }
    }
}

fn species_traits<'d, 't>(species: &Obj<'d, 't>) -> Option<Vec<String>> {
    for (key, _op, val) in species.fields() {
        if key.read_string() == "traits"
            && let Ok(obj) = val.read_object()
        {
            return Some(collect_repeated_key_values(&obj, "trait"));
        }
    }
    None
}

fn collect_repeated_key_values<'d, 't>(obj: &Obj<'d, 't>, key: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (k, _op, val) in obj.fields() {
        if k.read_string() == key
            && let Ok(s) = val.read_string()
        {
            out.push(s);
        }
    }
    out
}

fn collect_string_array<'d, 't>(reader: &Val<'d, 't>) -> Option<Vec<String>> {
    reader
        .read_array()
        .ok()?
        .values()
        .map(|entry| entry.read_string().ok())
        .collect()
}

fn read_bool(reader: &Val) -> Option<bool> {
    match reader.read_string().ok().as_deref() {
        Some("yes") => Some(true),
        Some("no") => Some(false),
        _ => None,
    }
}

fn normalise_country_type(raw: &str) -> String {
    match raw {
        "colony" => "default",
        other => other,
    }
    .to_string()
}

fn infer_archetype(traits: &[String]) -> &'static str {
    const MACHINE: &str = "MACHINE";
    const LITHOID: &str = "LITHOID";
    const BIOLOGICAL: &str = "BIOLOGICAL";
    if traits
        .iter()
        .any(|t| matches!(t.as_str(), "trait_machine_unit" | "trait_mechanical"))
    {
        MACHINE
    } else if traits.iter().any(|t| t == "trait_lithoid") {
        LITHOID
    } else {
        BIOLOGICAL
    }
}
