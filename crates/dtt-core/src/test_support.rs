use crate::condition::{
    ComparisonOperator, CompiledCondition, Condition, EvaluationContext, Predicate,
    PredicateArgument, PredicateEvaluation, ScopeEvaluation,
};
use crate::technology::{Area, Catalog, Definition, Id, Prerequisites};

#[derive(Clone, Copy)]
pub(crate) struct Context;

impl EvaluationContext for Context {
    fn predicate(&self, predicate: &Predicate) -> PredicateEvaluation {
        match predicate.name.as_str() {
            "true" => PredicateEvaluation::fixed(true),
            "false" => PredicateEvaluation::fixed(false),
            "future" => PredicateEvaluation::deferred("future"),
            name => panic!("unexpected test predicate: {name}"),
        }
    }

    fn scope(&self, reference: &str) -> ScopeEvaluation<Self> {
        assert!(matches!(reference, "absent" | "absent?"));
        ScopeEvaluation {
            context: *self,
            presence: PredicateEvaluation::fixed(false),
            optional: reference.ends_with('?'),
        }
    }

    fn any_object(&self, collection: &str) -> ScopeEvaluation<Self> {
        assert_eq!(collection, "empty");
        self.scope("absent")
    }

    fn scripted(&self, _name: &str) -> Self {
        *self
    }
}

pub(crate) fn leaf(name: &str) -> Condition {
    Condition::Predicate(Predicate {
        name: name.into(),
        operator: ComparisonOperator::Equal,
        argument: PredicateArgument::None,
    })
}

pub(crate) fn compiled(condition: Condition) -> CompiledCondition {
    CompiledCondition { condition }
}

pub(crate) fn definition(id: &str) -> Definition {
    Definition {
        id: id.into(),
        area: Area::Physics,
        tier: 1,
        prerequisites: Prerequisites::default(),
        potential: None,
        is_dangerous: false,
        is_rare: false,
        technology_swaps: Vec::new(),
    }
}

pub(crate) fn catalog(definitions: impl IntoIterator<Item = Definition>) -> Catalog {
    let mut result = Catalog::default();
    for definition in definitions {
        result.insert(definition);
    }
    result
}

pub(crate) fn ids(names: &[&str]) -> Vec<Id> {
    names.iter().map(|name| Id::from(*name)).collect()
}
