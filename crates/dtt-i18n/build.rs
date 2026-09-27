#![forbid(unsafe_code)]
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use fluent_syntax::ast::{Entry, Expression, InlineExpression, Pattern, PatternElement};
use std::collections::{BTreeMap, BTreeSet};
fn variables(pattern: &Pattern<&str>, names: &mut BTreeSet<String>) {
    for element in &pattern.elements {
        if let PatternElement::Placeable { expression } = element {
            expression_vars(expression, names);
        }
    }
}
fn expression_vars(expression: &Expression<&str>, names: &mut BTreeSet<String>) {
    match expression {
        Expression::Inline(expr) => inline_vars(expr, names),
        Expression::Select { selector, variants } => {
            inline_vars(selector, names);
            for variant in variants {
                variables(&variant.value, names);
            }
        }
    }
}
fn inline_vars(expr: &InlineExpression<&str>, names: &mut BTreeSet<String>) {
    match expr {
        InlineExpression::VariableReference { id } => {
            names.insert(id.name.into());
        }
        InlineExpression::Placeable { expression } => expression_vars(expression, names),
        InlineExpression::FunctionReference { arguments, .. } => {
            for arg in &arguments.positional {
                inline_vars(arg, names);
            }
            for arg in &arguments.named {
                inline_vars(&arg.value, names);
            }
        }
        InlineExpression::TermReference {
            arguments: Some(arguments),
            ..
        } => {
            for arg in &arguments.positional {
                inline_vars(arg, names);
            }
            for arg in &arguments.named {
                inline_vars(&arg.value, names);
            }
        }
        _ => {}
    }
}
fn validate(locale: &str, domain: &str) -> BTreeMap<String, BTreeSet<String>> {
    let path = format!("locales/{locale}/{domain}.ftl");
    println!("cargo:rerun-if-changed={path}");
    let source = std::fs::read_to_string(&path).expect("embedded catalog must exist");
    let resource = FluentResource::try_new(source).expect("catalog must parse without Junk");
    let mut contract = BTreeMap::new();
    for entry in resource.entries() {
        if let Entry::Message(message) = entry {
            let pattern = message.value.as_ref().expect("message must have a value");
            assert!(!pattern.elements.is_empty(), "empty message");
            let mut names = BTreeSet::new();
            variables(pattern, &mut names);
            assert!(
                contract.insert(message.id.name.to_owned(), names).is_none(),
                "duplicate ID"
            );
        }
    }
    assert!(!contract.is_empty(), "catalog must not be empty");
    let mut bundle = FluentBundle::new(vec![locale.parse().expect("valid locale")]);
    bundle
        .add_resource(resource)
        .expect("no duplicate resources");
    for (id, names) in &contract {
        for count in [0, 1, 2, 1_000_000] {
            let mut args = FluentArgs::new();
            for name in names {
                if ["count", "tier", "limit", "index"].contains(&name.as_str()) {
                    args.set(name, count);
                } else {
                    args.set(name, "测试 \"C:\\data\\file\" $tech_id$ §H £physics£");
                }
            }
            let mut errors = Vec::new();
            let output = bundle.format_pattern(
                bundle.get_message(id).unwrap().value().unwrap(),
                Some(&args),
                &mut errors,
            );
            assert!(errors.is_empty(), "{path}/{id}: {errors:?}");
            assert!(!output.trim().is_empty(), "{path}/{id}: empty output");
        }
    }
    contract
}
fn main() {
    for domain in ["cli", "errors", "diagnostics", "report", "game"] {
        assert_eq!(
            validate("en", domain),
            validate("zh-Hans", domain),
            "{domain}: inconsistent message/parameter contracts"
        );
    }
}
