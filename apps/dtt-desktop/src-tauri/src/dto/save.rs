use dtt_application as application;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SaveLibrarySourceDto {
    Local,
    SteamCloud,
}

impl From<SaveLibrarySourceDto> for application::SaveLibrarySource {
    fn from(value: SaveLibrarySourceDto) -> Self {
        match value {
            SaveLibrarySourceDto::Local => Self::Local,
            SaveLibrarySourceDto::SteamCloud => Self::SteamCloud,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ScanSaveLibraryRequestDto {
    pub source: SaveLibrarySourceDto,
    #[ts(optional)]
    pub documents_dir: Option<String>,
    pub steam_libraries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InspectSaveRequestDto {
    pub save_file: String,
    #[ts(optional)]
    pub country_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SaveLibraryDto {
    pub diagnostics: Vec<SaveScanDiagnosticDto>,
    pub accounts: Vec<SaveAccountDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SaveAccountDto {
    pub campaigns: Vec<SaveCampaignDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SaveCampaignDto {
    pub key: String,
    pub saves: Vec<SaveIndexDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SaveIndexDto {
    pub path: String,
    pub file_name: String,
    #[ts(optional)]
    pub metadata: Option<SaveMetadataDto>,
    pub modified_at_millis: u64,
    pub file_size: u64,
    pub state: SaveIndexStateDto,
    #[ts(optional)]
    pub unavailable_reason: Option<SaveUnavailableReasonDto>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SaveIndexStateDto {
    Text,
    Binary,
    Corrupt,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SaveMetadataDto {
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InspectedSaveDto {
    #[ts(optional)]
    pub snapshot: Option<SnapshotDto>,
    pub player_countries: Vec<PlayerCountryCandidateDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PlayerCountryCandidateDto {
    pub country_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDto {
    pub ethics: Vec<String>,
    pub government: GovernmentDto,
    pub ascension_perks: Vec<String>,
    pub traditions: Vec<String>,
    pub founder_species: SpeciesDto,
    pub country_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GovernmentDto {
    #[ts(optional)]
    pub authority: Option<String>,
    #[ts(optional)]
    pub origin: Option<String>,
    pub civics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesDto {
    #[ts(optional)]
    pub archetype: Option<String>,
    pub traits: Vec<String>,
}

impl From<application::SaveLibrary> for SaveLibraryDto {
    fn from(value: application::SaveLibrary) -> Self {
        Self {
            diagnostics: value.diagnostics.into_iter().map(Into::into).collect(),
            accounts: value.accounts.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<application::SaveAccount> for SaveAccountDto {
    fn from(value: application::SaveAccount) -> Self {
        Self {
            campaigns: value.campaigns.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<application::SaveCampaign> for SaveCampaignDto {
    fn from(value: application::SaveCampaign) -> Self {
        Self {
            key: value.key,
            saves: value.saves.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<application::SaveIndex> for SaveIndexDto {
    fn from(value: application::SaveIndex) -> Self {
        Self {
            path: value.path.to_string_lossy().into_owned(),
            file_name: value.file_name,
            metadata: value.metadata.map(Into::into),
            modified_at_millis: value.modified_at_millis,
            file_size: value.file_size,
            state: match value.state {
                application::SaveIndexState::Text => SaveIndexStateDto::Text,
                application::SaveIndexState::Binary => SaveIndexStateDto::Binary,
                application::SaveIndexState::Corrupt => SaveIndexStateDto::Corrupt,
            },
            unavailable_reason: value.unavailable_reason.map(Into::into),
        }
    }
}

impl From<dtt_application::InspectedSave> for InspectedSaveDto {
    fn from(value: dtt_application::InspectedSave) -> Self {
        Self {
            snapshot: value.snapshot.map(Into::into),
            player_countries: value.player_countries.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<dtt_application::PlayerCountryCandidate> for PlayerCountryCandidateDto {
    fn from(value: dtt_application::PlayerCountryCandidate) -> Self {
        Self {
            country_id: value.country_id,
        }
    }
}

impl From<dtt_application::SaveMetadata> for SaveMetadataDto {
    fn from(value: dtt_application::SaveMetadata) -> Self {
        Self {
            name: value.name,
            date: value.date,
        }
    }
}

impl From<dtt_application::Snapshot> for SnapshotDto {
    fn from(value: dtt_application::Snapshot) -> Self {
        Self {
            ethics: value.ethics.unwrap_or_default(),
            government: GovernmentDto {
                authority: value.government.authority,
                origin: value.government.origin,
                civics: value.government.civics.unwrap_or_default(),
            },
            ascension_perks: value.ascension_perks,
            traditions: value.traditions,
            founder_species: SpeciesDto {
                archetype: value
                    .founder_species
                    .as_ref()
                    .and_then(|species| species.archetype.clone()),
                traits: value
                    .founder_species
                    .and_then(|species| species.traits)
                    .unwrap_or_default(),
            },
            country_type: value.country_type.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum SaveUnavailableReasonDto {
    Binary,
    Corrupt { technical_detail: String },
}
impl From<application::SaveUnavailableReason> for SaveUnavailableReasonDto {
    fn from(value: application::SaveUnavailableReason) -> Self {
        match value {
            application::SaveUnavailableReason::Binary => Self::Binary,
            application::SaveUnavailableReason::Corrupt { technical_detail } => {
                Self::Corrupt { technical_detail }
            }
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SaveScanDiagnosticDto {
    pub path: String,
    pub kind: SaveScanFailureKindDto,
    pub technical_detail: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SaveScanFailureKindDto {
    SteamUsers,
    SteamUserEntry,
    SaveRoot,
    CampaignEntry,
    Campaign,
    SaveEntry,
    FileMetadata,
}
impl From<application::SaveScanDiagnostic> for SaveScanDiagnosticDto {
    fn from(value: application::SaveScanDiagnostic) -> Self {
        use application::SaveScanFailureKind as K;
        Self {
            path: value.path.to_string_lossy().into_owned(),
            technical_detail: value.technical_detail,
            kind: match value.kind {
                K::SteamUsers => SaveScanFailureKindDto::SteamUsers,
                K::SteamUserEntry => SaveScanFailureKindDto::SteamUserEntry,
                K::SaveRoot => SaveScanFailureKindDto::SaveRoot,
                K::CampaignEntry => SaveScanFailureKindDto::CampaignEntry,
                K::Campaign => SaveScanFailureKindDto::Campaign,
                K::SaveEntry => SaveScanFailureKindDto::SaveEntry,
                K::FileMetadata => SaveScanFailureKindDto::FileMetadata,
            },
        }
    }
}
