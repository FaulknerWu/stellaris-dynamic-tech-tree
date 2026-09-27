use dtt_core::condition::ContextReason;
use dtt_core::condition::{PredicateEvaluation, ScopeEvaluation};

use super::AnalysisContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ObjectKind {
    Country,
    Species,
    Federation,
    Planet,
    Situation,
    Leader,
}

#[derive(Debug, Clone)]
pub(super) enum ObjectRef {
    OpeningCountry,
    FounderSpecies,
    Future {
        kind: ObjectKind,
        owner: Option<Box<ObjectRef>>,
    },
    Unresolved {
        kind: Option<ObjectKind>,
        reason: ContextReason,
    },
}

impl ObjectRef {
    pub(super) fn kind(&self) -> Option<ObjectKind> {
        match self {
            Self::OpeningCountry => Some(ObjectKind::Country),
            Self::FounderSpecies => Some(ObjectKind::Species),
            Self::Future { kind, .. } => Some(*kind),
            Self::Unresolved { kind, .. } => *kind,
        }
    }
}

pub(super) fn resolve<'a>(
    reference: &str,
    context: &AnalysisContext<'a>,
) -> ScopeEvaluation<AnalysisContext<'a>> {
    let optional = reference.ends_with('?');
    let path = reference.trim_end_matches('?');
    let mut current = context.current.clone();
    for part in path.split('.') {
        current = step(part, current, context);
    }
    let presence = presence(&current, reference, context);
    ScopeEvaluation {
        context: context.enter(current, reference),
        presence,
        optional,
    }
}

fn step(reference: &str, current: ObjectRef, context: &AnalysisContext<'_>) -> ObjectRef {
    let key = reference.to_ascii_lowercase();
    match key.as_str() {
        "this" => current,
        "root" => context.root.clone(),
        key if repeated_depth(key, "prev").is_some() => {
            let depth = repeated_depth(key, "prev").expect("前序引用已经识别");
            context
                .previous
                .len()
                .checked_sub(depth)
                .and_then(|index| context.previous.get(index))
                .cloned()
                .unwrap_or_else(|| unresolved(None, ContextReason::MissingPreviousScope))
        }
        key if repeated_depth(key, "from").is_some() => {
            unresolved(None, ContextReason::MissingEventSource)
        }
        "founder_species" | "owner_species" | "species" => match current {
            ObjectRef::OpeningCountry if context.world.snapshot.founder_species.is_some() => {
                ObjectRef::FounderSpecies
            }
            ObjectRef::OpeningCountry => unresolved(
                Some(ObjectKind::Species),
                ContextReason::MissingFounderSpecies,
            ),
            ObjectRef::Future {
                kind: ObjectKind::Country,
                ..
            } => future(ObjectKind::Species, None),
            _ => unresolved(
                Some(ObjectKind::Species),
                ContextReason::UnknownSpeciesRelation,
            ),
        },
        "owner" => match current {
            ObjectRef::Future {
                kind: ObjectKind::Planet | ObjectKind::Leader,
                owner: Some(owner),
            } => *owner,
            ObjectRef::Future {
                kind: ObjectKind::Planet | ObjectKind::Leader,
                ..
            } => future(ObjectKind::Country, None),
            _ => unresolved(Some(ObjectKind::Country), ContextReason::UnknownOwner),
        },
        "federation" if current.kind() == Some(ObjectKind::Country) => {
            future(ObjectKind::Federation, None)
        }
        "capital_scope" if current.kind() == Some(ObjectKind::Country) => {
            future(ObjectKind::Planet, Some(current))
        }
        "leader" if current.kind() == Some(ObjectKind::Country) => {
            future(ObjectKind::Leader, Some(current))
        }
        _ => unresolved(None, ContextReason::UnknownScope),
    }
}

fn repeated_depth(value: &str, unit: &str) -> Option<usize> {
    (!value.is_empty()
        && value.len().is_multiple_of(unit.len())
        && value
            .as_bytes()
            .chunks(unit.len())
            .all(|chunk| chunk == unit.as_bytes()))
    .then_some(value.len() / unit.len())
}

fn unresolved(kind: Option<ObjectKind>, reason: ContextReason) -> ObjectRef {
    ObjectRef::Unresolved { kind, reason }
}

fn presence(target: &ObjectRef, name: &str, context: &AnalysisContext<'_>) -> PredicateEvaluation {
    match target {
        ObjectRef::OpeningCountry | ObjectRef::FounderSpecies => PredicateEvaluation::fixed(true),
        ObjectRef::Future { .. } => PredicateEvaluation::deferred(name),
        ObjectRef::Unresolved { reason, .. } => context.unknown(name, reason.clone()),
    }
}

pub(super) fn collection<'a>(
    name: &str,
    context: &AnalysisContext<'a>,
) -> ScopeEvaluation<AnalysisContext<'a>> {
    let kind = match name {
        "any_country" => Some(ObjectKind::Country),
        "any_relation" | "any_neighbor_country" | "any_subject"
            if context.current.kind() == Some(ObjectKind::Country) =>
        {
            Some(ObjectKind::Country)
        }
        "any_owned_species" if context.current.kind() == Some(ObjectKind::Country) => {
            Some(ObjectKind::Species)
        }
        "any_owned_planet" if context.current.kind() == Some(ObjectKind::Country) => {
            Some(ObjectKind::Planet)
        }
        "any_owned_leader" if context.current.kind() == Some(ObjectKind::Country) => {
            Some(ObjectKind::Leader)
        }
        "any_situation" if context.current.kind() == Some(ObjectKind::Country) => {
            Some(ObjectKind::Situation)
        }
        _ => None,
    };
    let current = kind
        .map(|kind| {
            let owner = matches!(name, "any_owned_planet" | "any_owned_leader")
                .then(|| context.current.clone());
            future(kind, owner)
        })
        .unwrap_or_else(|| unresolved(None, ContextReason::UnknownCollection));
    let presence = presence(&current, name, context);
    ScopeEvaluation {
        context: context.enter(current, name),
        presence,
        optional: false,
    }
}

fn future(kind: ObjectKind, owner: Option<ObjectRef>) -> ObjectRef {
    ObjectRef::Future {
        kind,
        owner: owner.map(Box::new),
    }
}
