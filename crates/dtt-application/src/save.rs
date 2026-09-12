use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use dtt_core::empire::Snapshot;
use dtt_stellaris::save::{Gamestate, GamestateKind, SaveMetadata};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub use dtt_stellaris::save::PlayerCountryCandidate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectSaveRequest {
    pub save_file: PathBuf,
    pub country_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectedSave {
    pub snapshot: Option<Snapshot>,
    pub player_countries: Vec<PlayerCountryCandidate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveLibrarySource {
    Local,
    SteamCloud,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSaveLibraryRequest {
    pub source: SaveLibrarySource,
    pub documents_dir: Option<PathBuf>,
    pub steam_libraries: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveLibrary {
    pub accounts: Vec<SaveAccount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveAccount {
    pub campaigns: Vec<SaveCampaign>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveCampaign {
    pub key: String,
    pub saves: Vec<SaveIndex>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveIndex {
    pub path: PathBuf,
    pub file_name: String,
    pub metadata: Option<SaveMetadata>,
    pub modified_at_millis: u64,
    pub file_size: u64,
    pub state: SaveIndexState,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveIndexState {
    Text,
    Binary,
    Corrupt,
}

pub fn inspect_save(request: &InspectSaveRequest) -> Result<InspectedSave> {
    let gamestate = open_text_save(&request.save_file)?;
    let player_countries = dtt_stellaris::save::player_country_candidates(&gamestate)?;
    let country_id = match request.country_id {
        Some(country_id) => Some(country_id),
        None if player_countries.len() == 1 => Some(player_countries[0].country_id),
        None if player_countries.is_empty() => {
            return Err(Error::PlayerCountryMissing);
        }
        None => None,
    };
    let snapshot = country_id
        .map(|country_id| dtt_stellaris::save::extract_snapshot_for_country(&gamestate, country_id))
        .transpose()?;
    Ok(InspectedSave {
        snapshot,
        player_countries,
    })
}

pub fn scan_save_library(request: &ScanSaveLibraryRequest) -> Result<SaveLibrary> {
    let scanned = match request.source {
        SaveLibrarySource::Local => {
            let documents_dir = request.documents_dir.as_deref().ok_or_else(|| {
                Error::Settings("documents_dir is required for local save scanning".into())
            })?;
            dtt_stellaris::save::scan_local(documents_dir)
        }
        SaveLibrarySource::SteamCloud => dtt_stellaris::save::scan_cloud(&request.steam_libraries),
    };

    let mut grouped = BTreeMap::<Option<String>, BTreeMap<String, Vec<SaveIndex>>>::new();
    for save in scanned.saves {
        let state = match save.gamestate_kind {
            Some(GamestateKind::Text) => SaveIndexState::Text,
            Some(GamestateKind::Binary) => SaveIndexState::Binary,
            None => SaveIndexState::Corrupt,
        };
        grouped
            .entry(save.steam_user_id)
            .or_default()
            .entry(save.campaign_key)
            .or_default()
            .push(SaveIndex {
                path: save.path,
                file_name: save.file_name,
                metadata: save.metadata,
                modified_at_millis: save.modified_at_millis,
                file_size: save.file_size,
                state,
                unavailable_reason: save.unavailable_reason,
            });
    }

    let mut accounts = grouped
        .into_iter()
        .map(|(steam_user_id, campaigns)| {
            let mut campaigns = campaigns
                .into_iter()
                .map(|(key, mut saves)| {
                    saves.sort_by(compare_save_index);
                    SaveCampaign { key, saves }
                })
                .collect::<Vec<_>>();
            campaigns.sort_by(|left, right| {
                compare_optional_save(left.saves.first(), right.saves.first())
                    .then_with(|| left.key.cmp(&right.key))
            });
            (steam_user_id, SaveAccount { campaigns })
        })
        .collect::<Vec<_>>();
    accounts.sort_by(|left, right| left.0.cmp(&right.0));

    Ok(SaveLibrary {
        accounts: accounts.into_iter().map(|(_, account)| account).collect(),
    })
}

pub(crate) fn open_text_save(path: &Path) -> Result<Vec<u8>> {
    if !path.is_file() {
        return Err(Error::SaveUnavailable(path.to_path_buf()));
    }
    let archive = dtt_stellaris::save::open(path)?;
    match archive.gamestate {
        Gamestate::Text(bytes) => Ok(bytes),
        Gamestate::Binary => Err(Error::UnsupportedBinarySave),
    }
}

fn compare_save_index(left: &SaveIndex, right: &SaveIndex) -> std::cmp::Ordering {
    game_date_key(right)
        .cmp(&game_date_key(left))
        .then_with(|| right.modified_at_millis.cmp(&left.modified_at_millis))
        .then_with(|| left.file_name.cmp(&right.file_name))
}

fn compare_optional_save(
    left: Option<&SaveIndex>,
    right: Option<&SaveIndex>,
) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => compare_save_index(left, right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn game_date_key(save: &SaveIndex) -> Vec<u64> {
    save.metadata
        .as_ref()
        .and_then(|metadata| metadata.date.as_deref())
        .map(|date| {
            date.split('.')
                .map(|part| part.parse::<u64>().unwrap_or_default())
                .collect()
        })
        .unwrap_or_default()
}
