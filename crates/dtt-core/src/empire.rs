use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub ethics: Option<Vec<String>>,
    pub government: Government,
    pub ascension_perks: Vec<String>,
    pub traditions: Vec<String>,
    pub graphical_culture: Option<String>,
    pub founder_species: Option<Species>,
    pub country_type: Option<String>,
    pub is_nomadic: Option<bool>,
    pub is_ai: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Government {
    pub authority: Option<String>,
    pub origin: Option<String>,
    pub government_type: Option<String>,
    pub civics: Option<Vec<String>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Species {
    pub archetype: Option<String>,
    pub traits: Option<Vec<String>>,
}
