use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GenerationStage {
    SaveParse,
    LoadOrder,
    IngestTech,
    Relations,
    IngestL10n,
    Render,
    Cycles,
    WriteOutput,
    Done,
}

impl GenerationStage {
    pub fn percent(self) -> u8 {
        match self {
            GenerationStage::SaveParse => 5,
            GenerationStage::LoadOrder => 20,
            GenerationStage::IngestTech => 30,
            GenerationStage::Relations => 35,
            GenerationStage::IngestL10n => 45,
            GenerationStage::Render => 50,
            GenerationStage::Cycles => 60,
            GenerationStage::WriteOutput => 80,
            GenerationStage::Done => 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenerationStatus {
    Success,
    Incomplete,
}
