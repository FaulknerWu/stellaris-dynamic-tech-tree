use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;

use crate::technology::{Catalog, Id, Prerequisites};

pub struct Graph {
    unlocks: BTreeMap<Id, Vec<Id>>,
    prerequisites: BTreeMap<Id, Prerequisites>,
    cycles: CycleReport,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CycleReport {
    pub self_references: Vec<Id>,
    pub complex_cycles: Vec<Vec<Id>>,
}

impl Graph {
    pub fn build(catalog: &Catalog, eligible: &[Id]) -> Graph {
        let nodes: BTreeSet<Id> = eligible.iter().cloned().collect();
        let normalised = catalog.normalised_prerequisites();

        let mut unlocks: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
        let mut prerequisites: BTreeMap<Id, Prerequisites> = BTreeMap::new();
        for id in &nodes {
            unlocks.entry(id.clone()).or_default();
            prerequisites.entry(id.clone()).or_default();
        }

        for id in &nodes {
            let requirements = normalised
                .get(id)
                .expect("eligible technology id must exist in the catalog")
                .clone();

            for p in requirements.candidates() {
                if nodes.contains(p) {
                    unlocks.entry(p.clone()).or_default().push(id.clone());
                }
            }
            prerequisites.insert(id.clone(), requirements);
        }

        for v in unlocks.values_mut() {
            v.sort();
            v.dedup();
        }

        let cycles = detect_cycles(&nodes, &unlocks);
        Graph {
            unlocks,
            prerequisites,
            cycles,
        }
    }

    pub(crate) fn unlocks(&self, id: &Id) -> &[Id] {
        self.unlocks.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub(crate) fn prerequisites(&self, id: &Id) -> Option<&Prerequisites> {
        self.prerequisites.get(id)
    }

    pub fn cycle_report(&self) -> &CycleReport {
        &self.cycles
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Color {
    White,
    Gray,
    Black,
}

fn detect_cycles(nodes: &BTreeSet<Id>, unlocks: &BTreeMap<Id, Vec<Id>>) -> CycleReport {
    let mut self_refs: Vec<Id> = nodes
        .iter()
        .filter(|id| unlocks.get(*id).is_some_and(|v| v.contains(id)))
        .cloned()
        .collect();
    self_refs.sort();

    let mut complex: BTreeSet<Vec<Id>> = BTreeSet::new();
    let mut color: HashMap<Id, Color> = HashMap::new();
    let mut stack: Vec<Id> = Vec::new();

    for start in nodes {
        if !color.contains_key(start) {
            dfs_cycle(start, unlocks, &mut color, &mut stack, &mut complex);
        }
    }

    CycleReport {
        self_references: self_refs,
        complex_cycles: complex.into_iter().collect(),
    }
}

fn dfs_cycle(
    node: &Id,
    unlocks: &BTreeMap<Id, Vec<Id>>,
    color: &mut HashMap<Id, Color>,
    stack: &mut Vec<Id>,
    complex: &mut BTreeSet<Vec<Id>>,
) {
    color.insert(node.clone(), Color::Gray);
    stack.push(node.clone());

    for next in unlocks.get(node).into_iter().flatten() {
        if next == node {
            continue;
        }
        match color.get(next).copied().unwrap_or(Color::White) {
            Color::Gray => {
                let pos = stack.iter().position(|x| x == next).unwrap_or(0);
                let cycle: Vec<Id> = stack[pos..].to_vec();
                complex.insert(normalise_cycle(cycle));
            }
            Color::White => dfs_cycle(next, unlocks, color, stack, complex),
            Color::Black => {}
        }
    }

    color.insert(node.clone(), Color::Black);
    stack.pop();
}

fn normalise_cycle(mut cycle: Vec<Id>) -> Vec<Id> {
    if cycle.len() <= 1 {
        return cycle;
    }
    let min_pos = cycle
        .iter()
        .enumerate()
        .min_by_key(|(_, id)| *id)
        .map(|(i, _)| i)
        .unwrap_or(0);
    cycle.rotate_left(min_pos);
    cycle
}
