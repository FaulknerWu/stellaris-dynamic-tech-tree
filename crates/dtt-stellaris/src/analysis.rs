use dtt_core::condition::{
    EvaluationContext, GraphicalCultures, Predicate, PredicateEvaluation, ScopeEvaluation,
    UnknownConditionReason,
};
use dtt_core::empire::Snapshot;

mod policy;
mod scope;

use scope::ObjectRef;

pub struct World<'a> {
    pub snapshot: &'a Snapshot,
    pub ship_kinds: &'a GraphicalCultures,
}

#[derive(Clone)]
pub struct AnalysisContext<'a> {
    world: &'a World<'a>,
    current: ObjectRef,
    root: ObjectRef,
    previous: Vec<ObjectRef>,
    path: Vec<String>,
    calls: Vec<String>,
}

impl<'a> World<'a> {
    pub fn context(&'a self) -> AnalysisContext<'a> {
        AnalysisContext {
            world: self,
            current: ObjectRef::OpeningCountry,
            root: ObjectRef::OpeningCountry,
            previous: Vec::new(),
            path: Vec::new(),
            calls: Vec::new(),
        }
    }
}

impl AnalysisContext<'_> {
    fn unknown(&self, trigger: &str, detail: impl Into<String>) -> PredicateEvaluation {
        let mut detail = detail.into();
        if !self.path.is_empty() {
            detail.push_str(&format!("；作用域路径：{}", self.path.join(" → ")));
        }
        if !self.calls.is_empty() {
            detail.push_str(&format!("；脚本调用：{}", self.calls.join(" → ")));
        }
        PredicateEvaluation::unknown(UnknownConditionReason::Context {
            trigger: trigger.into(),
            detail,
        })
    }

    fn enter(&self, current: ObjectRef, reference: &str) -> Self {
        let mut next = self.clone();
        next.previous.push(self.current.clone());
        next.current = current;
        next.path.push(reference.to_string());
        next
    }
}

impl EvaluationContext for AnalysisContext<'_> {
    fn predicate(&self, predicate: &Predicate) -> PredicateEvaluation {
        policy::evaluate(predicate, self)
    }
    fn scope(&self, reference: &str) -> ScopeEvaluation<Self> {
        scope::resolve(reference, self)
    }
    fn any_object(&self, collection: &str) -> ScopeEvaluation<Self> {
        scope::collection(collection, self)
    }
    fn scripted(&self, name: &str) -> Self {
        let mut next = self.clone();
        next.calls.push(name.to_string());
        next
    }
}
