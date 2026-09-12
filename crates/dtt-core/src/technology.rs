use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};

use crate::condition::CompiledCondition;

mod eligibility;
mod swap;

pub use eligibility::{
    DeferredTriggerDiagnostic, EligibilityReport, UnknownTriggerDiagnostic, evaluate_eligibility,
};
pub use swap::{SwapEntry, SwapOutcome, SwapResolution, SwapUnknownStrategy, resolve_swaps};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id(Arc<str>);

impl Id {
    pub fn new(id: impl Into<Box<str>>) -> Self {
        Self(Arc::from(id.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for Id {
    fn from(s: &str) -> Self {
        Id::new(s)
    }
}

impl From<String> for Id {
    fn from(s: String) -> Self {
        Id::new(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub id: Id,
    pub area: Area,
    pub tier: i32,
    pub prerequisites: Prerequisites,
    pub potential: Option<CompiledCondition>,
    pub is_dangerous: bool,
    pub is_rare: bool,
    pub technology_swaps: Vec<SwapVariant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Area {
    Physics,
    Society,
    Engineering,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prerequisites {
    pub all_of: Vec<Id>,
    pub any_of_groups: Vec<Vec<Id>>,
}

impl Prerequisites {
    fn normalised(&self, aliases: &HashMap<Id, Id>) -> Self {
        let resolve = |id: &Id| aliases.get(id).unwrap_or(id).clone();

        let mut all_of: Vec<Id> = self.all_of.iter().map(&resolve).collect();
        all_of.sort();
        all_of.dedup();

        let mut any_of_groups: Vec<Vec<Id>> = self
            .any_of_groups
            .iter()
            .map(|group| {
                let mut resolved: Vec<Id> = group.iter().map(&resolve).collect();
                resolved.sort();
                resolved.dedup();
                resolved
            })
            .filter(|group| !group.is_empty())
            .collect();
        any_of_groups.sort();
        any_of_groups.dedup();

        Self {
            all_of,
            any_of_groups,
        }
    }

    pub(crate) fn candidates(&self) -> impl Iterator<Item = &Id> {
        self.all_of
            .iter()
            .chain(self.any_of_groups.iter().flatten())
    }

    pub(crate) fn remaining_after(&self, satisfied_by: &Id) -> Self {
        Self {
            all_of: self
                .all_of
                .iter()
                .filter(|prerequisite| *prerequisite != satisfied_by)
                .cloned()
                .collect(),
            any_of_groups: self
                .any_of_groups
                .iter()
                .filter(|group| !group.contains(satisfied_by))
                .cloned()
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapVariant {
    pub active_id: Id,
    pub trigger: CompiledCondition,
    pub area: Option<Area>,
}

#[derive(Debug, Default, Clone)]
pub struct Catalog {
    technologies: HashMap<Id, Definition>,
    normalised_prerequisites: OnceLock<HashMap<Id, Prerequisites>>,
}

impl Catalog {
    pub fn insert(&mut self, technology: Definition) -> Option<Definition> {
        self.normalised_prerequisites.take();
        self.technologies.insert(technology.id.clone(), technology)
    }

    pub fn get(&self, id: &Id) -> Option<&Definition> {
        self.technologies.get(id)
    }

    pub fn contains(&self, id: &Id) -> bool {
        self.technologies.contains_key(id)
    }

    pub fn values(&self) -> impl Iterator<Item = &Definition> {
        self.technologies.values()
    }

    pub fn len(&self) -> usize {
        self.technologies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.technologies.is_empty()
    }

    pub(crate) fn sorted_ids(&self) -> Vec<Id> {
        let mut ids: Vec<_> = self.technologies.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub(crate) fn normalised_prerequisites(&self) -> &HashMap<Id, Prerequisites> {
        self.normalised_prerequisites
            .get_or_init(|| self.compute_normalised_prerequisites())
    }

    fn compute_normalised_prerequisites(&self) -> HashMap<Id, Prerequisites> {
        let aliases = self.prerequisite_aliases();
        self.technologies
            .iter()
            .map(|(id, technology)| (id.clone(), technology.prerequisites.normalised(&aliases)))
            .collect()
    }

    fn prerequisite_aliases(&self) -> HashMap<Id, Id> {
        let mut owners: BTreeMap<Id, BTreeSet<Id>> = BTreeMap::new();
        for technology in self.technologies.values() {
            for swap in &technology.technology_swaps {
                if !self.technologies.contains_key(&swap.active_id) {
                    owners
                        .entry(swap.active_id.clone())
                        .or_default()
                        .insert(technology.id.clone());
                }
            }
        }
        owners
            .into_iter()
            .filter_map(|(active_id, base_ids)| {
                (base_ids.len() == 1).then(|| (active_id, base_ids.into_iter().next().unwrap()))
            })
            .collect()
    }
}
