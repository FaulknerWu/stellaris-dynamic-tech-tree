mod container;
mod library;
mod metadata;
mod snapshot;

pub(crate) use container::index;
pub use container::open;
pub use library::{SaveScanDiagnostic, SaveScanResult, ScannedSave, scan_cloud, scan_local};
pub use metadata::SaveMetadata;
pub use snapshot::{
    PlayerCountryCandidate, extract_snapshot_for_country, player_country_candidates,
};

#[derive(Debug, Clone)]
pub struct Archive {
    pub metadata: SaveMetadata,
    pub gamestate: Gamestate,
}

#[derive(Debug, Clone)]
pub(crate) struct SaveContainerIndex {
    pub metadata: SaveMetadata,
    pub gamestate_kind: GamestateKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamestateKind {
    Text,
    Binary,
}

#[derive(Debug, Clone)]
pub enum Gamestate {
    Text(Vec<u8>),
    Binary,
}
