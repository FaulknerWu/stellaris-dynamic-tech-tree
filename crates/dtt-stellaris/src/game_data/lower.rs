use std::collections::{HashMap, HashSet};

use dtt_core::condition::{
    ComparisonOperator, CompiledCondition, Condition, ConditionBranch, Predicate,
    PredicateArgument, UnknownConditionReason,
};

use crate::clausewitz::script::{Script, ScriptArgument, ScriptParameter};
use crate::clausewitz::value::{resolve_at_variable, substitute_placeholders};

pub(crate) struct LowerInputs<'a> {
    pub scripted: &'a HashMap<String, Script>,
    pub scripted_names: &'a HashSet<String>,
    pub variables: &'a HashMap<String, String>,
}

struct LowerCtx<'inputs, 'reachable> {
    inputs: &'inputs LowerInputs<'inputs>,
    reachable_scripted: &'reachable mut HashSet<String>,
    parameters: HashMap<String, String>,
    stack: HashSet<String>,
}

pub(crate) fn lower_script(
    script: &Script,
    inputs: &LowerInputs<'_>,
    reachable_scripted: &mut HashSet<String>,
) -> CompiledCondition {
    let mut ctx = LowerCtx {
        inputs,
        reachable_scripted,
        parameters: HashMap::new(),
        stack: HashSet::new(),
    };
    CompiledCondition {
        condition: lower_node(script, &mut ctx),
    }
}

fn lower_node(script: &Script, ctx: &mut LowerCtx<'_, '_>) -> Condition {
    match script {
        Script::All(items) => {
            Condition::All(items.iter().map(|item| lower_node(item, ctx)).collect())
        }
        Script::Any(items) => {
            Condition::Any(items.iter().map(|item| lower_node(item, ctx)).collect())
        }
        Script::Not(inner) => Condition::Not(Box::new(lower_node(inner, ctx))),
        Script::If { branches, fallback } => Condition::If {
            branches: branches
                .iter()
                .map(|branch| ConditionBranch {
                    limit: lower_node(&branch.limit, ctx),
                    body: lower_node(&branch.body, ctx),
                })
                .collect(),
            fallback: fallback
                .as_ref()
                .map(|body| Box::new(lower_node(body, ctx))),
        },
        Script::Scope { reference, inner } => match bind_text(reference, ctx) {
            Some(reference) => Condition::Scope {
                reference,
                inner: Box::new(lower_node(inner, ctx)),
            },
            None => malformed(reference),
        },
        Script::AnyObject { collection, inner } => match bind_text(collection, ctx) {
            Some(collection) => Condition::AnyObject {
                collection,
                inner: Box::new(lower_node(inner, ctx)),
            },
            None => malformed(collection),
        },
        Script::Call {
            name,
            operator,
            argument,
        } => lower_call(name, *operator, argument, ctx),
    }
}

fn lower_call(
    name: &str,
    operator: ComparisonOperator,
    argument: &ScriptArgument,
    ctx: &mut LowerCtx<'_, '_>,
) -> Condition {
    let Some(name) = bind_text(name, ctx) else {
        return malformed(name);
    };
    if ctx.inputs.scripted_names.contains(&name) {
        ctx.reachable_scripted.insert(name.clone());
    }
    let Some(argument) = bind_argument(argument, ctx) else {
        return malformed(&name);
    };
    if ctx.inputs.scripted.contains_key(&name) {
        return lower_scripted(&name, operator, &argument, ctx);
    }
    let argument = match argument {
        ScriptArgument::None => PredicateArgument::None,
        ScriptArgument::Scalar(value) => PredicateArgument::Scalar(value),
        ScriptArgument::Parameters(fields) => PredicateArgument::Parameters(
            fields
                .into_iter()
                .map(|field| (field.name, field.value))
                .collect(),
        ),
        ScriptArgument::Structured { keys } => PredicateArgument::Structured { keys },
        ScriptArgument::Malformed => PredicateArgument::Malformed,
    };
    Condition::Predicate(Predicate {
        name,
        operator,
        argument,
    })
}

fn lower_scripted(
    name: &str,
    operator: ComparisonOperator,
    argument: &ScriptArgument,
    ctx: &mut LowerCtx<'_, '_>,
) -> Condition {
    if !matches!(
        operator,
        ComparisonOperator::Equal | ComparisonOperator::NotEqual
    ) {
        return Condition::Unknown(UnknownConditionReason::Operator {
            trigger: name.to_string(),
            operator,
        });
    }
    let Some(provided) = scripted_parameters(argument) else {
        return malformed(name);
    };
    let body = ctx.inputs.scripted.get(name).expect("脚本定义已确认存在");
    if !parameters_are_bound(body, &provided) {
        return malformed(name);
    }
    if !ctx.stack.insert(name.to_string()) {
        return Condition::Unknown(UnknownConditionReason::Context {
            trigger: name.to_string(),
            reason: dtt_core::condition::ContextReason::RecursiveScript,
            scope_path: Vec::new(),
            calls: Vec::new(),
        });
    }
    let previous = std::mem::replace(&mut ctx.parameters, provided);
    let body = lower_node(body, ctx);
    ctx.parameters = previous;
    ctx.stack.remove(name);
    let call = Condition::Scripted {
        name: name.to_string(),
        inner: Box::new(body),
    };
    if matches!(argument, ScriptArgument::Scalar(value) if value == "no")
        ^ (operator == ComparisonOperator::NotEqual)
    {
        Condition::Not(Box::new(call))
    } else {
        call
    }
}

fn malformed(name: &str) -> Condition {
    Condition::Unknown(UnknownConditionReason::MalformedArgument(name.to_string()))
}

fn bind_text(value: &str, ctx: &LowerCtx<'_, '_>) -> Option<String> {
    let substituted = substitute_placeholders(value, &ctx.parameters)?;
    Some(resolve_at_variable(&substituted, ctx.inputs.variables))
}

fn bind_argument(argument: &ScriptArgument, ctx: &LowerCtx<'_, '_>) -> Option<ScriptArgument> {
    match argument {
        ScriptArgument::None => Some(ScriptArgument::None),
        ScriptArgument::Malformed => Some(ScriptArgument::Malformed),
        ScriptArgument::Structured { keys } => {
            Some(ScriptArgument::Structured { keys: keys.clone() })
        }
        ScriptArgument::Scalar(value) => bind_text(value, ctx).map(ScriptArgument::Scalar),
        ScriptArgument::Parameters(parameters) => parameters
            .iter()
            .map(|parameter| {
                Some(ScriptParameter {
                    name: bind_text(&parameter.name, ctx)?,
                    value: bind_text(&parameter.value, ctx)?,
                })
            })
            .collect::<Option<Vec<_>>>()
            .map(ScriptArgument::Parameters),
    }
}

fn scripted_parameters(argument: &ScriptArgument) -> Option<HashMap<String, String>> {
    match argument {
        ScriptArgument::Scalar(value) if value == "yes" || value == "no" => Some(HashMap::new()),
        ScriptArgument::None | ScriptArgument::Scalar(_) => None,
        ScriptArgument::Parameters(fields) => Some(
            fields
                .iter()
                .map(|parameter| (parameter.name.clone(), parameter.value.clone()))
                .collect(),
        ),
        ScriptArgument::Malformed | ScriptArgument::Structured { .. } => None,
    }
}

fn parameters_are_bound(script: &Script, parameters: &HashMap<String, String>) -> bool {
    match script {
        Script::All(items) | Script::Any(items) => items
            .iter()
            .all(|item| parameters_are_bound(item, parameters)),
        Script::Not(inner) => parameters_are_bound(inner, parameters),
        Script::Scope { reference, inner } => {
            substitute_placeholders(reference, parameters).is_some()
                && parameters_are_bound(inner, parameters)
        }
        Script::AnyObject { collection, inner } => {
            substitute_placeholders(collection, parameters).is_some()
                && parameters_are_bound(inner, parameters)
        }
        Script::If { branches, fallback } => {
            branches.iter().all(|branch| {
                parameters_are_bound(&branch.limit, parameters)
                    && parameters_are_bound(&branch.body, parameters)
            }) && fallback
                .as_ref()
                .is_none_or(|fallback| parameters_are_bound(fallback, parameters))
        }
        Script::Call { name, argument, .. } => {
            substitute_placeholders(name, parameters).is_some()
                && argument_placeholders_bound(argument, parameters)
        }
    }
}

fn argument_placeholders_bound(
    argument: &ScriptArgument,
    parameters: &HashMap<String, String>,
) -> bool {
    match argument {
        ScriptArgument::None | ScriptArgument::Malformed | ScriptArgument::Structured { .. } => {
            true
        }
        ScriptArgument::Scalar(value) => substitute_placeholders(value, parameters).is_some(),
        ScriptArgument::Parameters(fields) => fields.iter().all(|parameter| {
            substitute_placeholders(&parameter.name, parameters).is_some()
                && substitute_placeholders(&parameter.value, parameters).is_some()
        }),
    }
}
