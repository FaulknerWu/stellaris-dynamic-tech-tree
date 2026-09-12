use std::collections::{HashMap, HashSet};
use std::ops::Not;

use serde::{Deserialize, Serialize};

mod evaluation;

pub use evaluation::{DeferredTriggerOccurrence, TriggerEvaluation, UnknownTriggerOccurrence};

pub type GraphicalCultures = HashMap<String, HashSet<String>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TruthValue {
    True,
    False,
    Unknown,
}

impl TruthValue {
    pub fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl Not for TruthValue {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }
}

impl From<bool> for TruthValue {
    fn from(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    Equal,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    NotEqual,
    Exact,
    Exists,
}

impl ComparisonOperator {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Equal => "=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::NotEqual => "!=",
            Self::Exact => "==",
            Self::Exists => "?=",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownStrategy {
    #[default]
    IncludeFlagged,
    ExcludeStrict,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UnknownConditionReason {
    Trigger(String),
    Operator {
        trigger: String,
        operator: ComparisonOperator,
    },
    MalformedArgument(String),
    StructuredArgument {
        trigger: String,
        keys: Vec<String>,
    },
    Context {
        trigger: String,
        detail: String,
    },
}

impl UnknownConditionReason {
    pub fn trigger_name(&self) -> &str {
        match self {
            Self::Trigger(name) | Self::MalformedArgument(name) => name,
            Self::Operator { trigger, .. }
            | Self::StructuredArgument { trigger, .. }
            | Self::Context { trigger, .. } => trigger,
        }
    }
}

impl std::fmt::Display for UnknownConditionReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trigger(name) => write!(formatter, "未识别触发器 `{name}`"),
            Self::Operator { trigger, operator } => write!(
                formatter,
                "触发器 `{trigger}` 不支持运算符 `{}`",
                operator.symbol()
            ),
            Self::MalformedArgument(name) => write!(formatter, "触发器 `{name}` 的参数无效"),
            Self::StructuredArgument { trigger, keys } => write!(
                formatter,
                "触发器 `{trigger}` 的结构化参数尚未支持：{}",
                keys.join(", ")
            ),
            Self::Context { trigger, detail } => write!(formatter, "{trigger}：{detail}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
    If {
        branches: Vec<ConditionBranch>,
        fallback: Option<Box<Condition>>,
    },
    Scope {
        reference: String,
        inner: Box<Condition>,
    },
    AnyObject {
        collection: String,
        inner: Box<Condition>,
    },
    Scripted {
        name: String,
        inner: Box<Condition>,
    },
    Predicate(Predicate),
    Unknown(UnknownConditionReason),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionBranch {
    pub limit: Condition,
    pub body: Condition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Predicate {
    pub name: String,
    pub operator: ComparisonOperator,
    pub argument: PredicateArgument,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PredicateArgument {
    None,
    Scalar(String),
    Parameters(Vec<(String, String)>),
    Structured { keys: Vec<String> },
    Malformed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TruthBounds {
    pub guaranteed: TruthValue,
    pub possible: TruthValue,
}

impl TruthBounds {
    pub fn fixed(value: bool) -> Self {
        Self {
            guaranteed: value.into(),
            possible: value.into(),
        }
    }

    pub fn deferred() -> Self {
        Self {
            guaranteed: TruthValue::False,
            possible: TruthValue::True,
        }
    }

    pub fn unknown() -> Self {
        Self {
            guaranteed: TruthValue::Unknown,
            possible: TruthValue::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PredicateEvaluation {
    pub bounds: TruthBounds,
    pub unknown: Option<UnknownConditionReason>,
    pub deferred: Option<String>,
}

impl PredicateEvaluation {
    pub fn fixed(value: bool) -> Self {
        Self {
            bounds: TruthBounds::fixed(value),
            unknown: None,
            deferred: None,
        }
    }

    pub fn deferred(name: impl Into<String>) -> Self {
        Self {
            bounds: TruthBounds::deferred(),
            unknown: None,
            deferred: Some(name.into()),
        }
    }

    pub fn unknown(reason: UnknownConditionReason) -> Self {
        Self {
            bounds: TruthBounds::unknown(),
            unknown: Some(reason),
            deferred: None,
        }
    }
}

pub struct ScopeEvaluation<C> {
    pub context: C,
    pub presence: PredicateEvaluation,
    pub optional: bool,
}

pub trait EvaluationContext: Sized {
    fn predicate(&self, predicate: &Predicate) -> PredicateEvaluation;
    fn scope(&self, reference: &str) -> ScopeEvaluation<Self>;
    fn any_object(&self, collection: &str) -> ScopeEvaluation<Self>;
    fn scripted(&self, name: &str) -> Self;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledCondition {
    pub condition: Condition,
}

impl CompiledCondition {
    pub fn always() -> Self {
        Self {
            condition: Condition::All(Vec::new()),
        }
    }

    pub fn evaluate_detailed(&self, context: &impl EvaluationContext) -> TriggerEvaluation {
        evaluation::evaluate_detailed(&self.condition, context)
    }
}

impl Default for CompiledCondition {
    fn default() -> Self {
        Self::always()
    }
}
