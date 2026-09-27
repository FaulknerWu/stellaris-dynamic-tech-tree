use super::GenerationReport;
use dtt_i18n::{AppLocale, Domain, ReportLabel, TranslationError, Translator};

/// The locale is captured at task creation, independent of later interface changes.
pub fn render_report(
    report: &GenerationReport,
    locale: AppLocale,
) -> Result<String, TranslationError> {
    let t = Translator::new(locale, Domain::Report)?;
    let diagnostics = Translator::new(locale, Domain::Diagnostics)?;
    let technical = t.report_label(ReportLabel::Technical)?;
    let mut output = format!("=== {} ===\n\n", t.report_label(ReportLabel::Title)?);
    let unknown = report
        .unknown_triggers
        .iter()
        .map(|d| {
            Ok(format!(
                "{}: {} ×{}: {}",
                d.reason.trigger_name(),
                render_condition(&d.reason, &diagnostics)?,
                d.occurrences,
                d.tech_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })
        .collect::<Result<Vec<_>, TranslationError>>()?;
    let game_data = report
        .game_data_diagnostics
        .iter()
        .map(|d| {
            let issue = render_issue(&d.issue, &diagnostics)?;
            Ok(format!(
                "{} / {} / {} / {} / {}\n    {}\n    {technical}: {}",
                d.source,
                d.category.as_str(),
                d.kind.as_str(),
                d.subject.as_deref().unwrap_or("—"),
                d.byte_offset
                    .map(|offset| offset.to_string())
                    .unwrap_or_else(|| "—".into()),
                issue,
                d.technical_detail.as_deref().unwrap_or("—")
            ))
        })
        .collect::<Result<Vec<_>, TranslationError>>()?;
    let sections = [
        (
            ReportLabel::MissingMods,
            report.missing_mod_descriptors.clone(),
        ),
        (
            ReportLabel::Eligible,
            report.eligible.iter().map(ToString::to_string).collect(),
        ),
        (
            ReportLabel::Uncertain,
            report.uncertain.iter().map(ToString::to_string).collect(),
        ),
        (
            ReportLabel::ExcludedIdentity,
            report
                .excluded_identity
                .iter()
                .map(ToString::to_string)
                .collect(),
        ),
        (
            ReportLabel::ExcludedPrerequisite,
            report
                .excluded_prereq
                .iter()
                .map(|(id, missing)| {
                    format!(
                        "{id}: {}",
                        missing
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect(),
        ),
        (ReportLabel::Unknown, unknown),
        (
            ReportLabel::Deferred,
            report
                .deferred_triggers
                .iter()
                .map(|d| {
                    format!(
                        "{} ×{}: {}",
                        d.name,
                        d.occurrences,
                        d.tech_ids
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect(),
        ),
        (ReportLabel::GameData, game_data),
        (
            ReportLabel::Definitions,
            report
                .unhandled_definition_fields
                .iter()
                .map(|d| {
                    format!(
                        "{} / {}: {}",
                        d.source,
                        d.technology_id,
                        d.fields.join(", ")
                    )
                })
                .collect(),
        ),
        (
            ReportLabel::Localisation,
            report
                .localisation_diagnostics
                .iter()
                .map(|d| {
                    Ok(format!(
                        "{} / {} / {}: {}\n    {technical}: {}",
                        d.language,
                        d.source,
                        d.path,
                        diagnostics.localisation_failure(matches!(
                            d.kind,
                            crate::LocalisationFailureKind::InvalidUtf8
                        ))?,
                        d.technical_detail
                    ))
                })
                .collect::<Result<Vec<_>, TranslationError>>()?,
        ),
        (
            ReportLabel::SelfCycles,
            report
                .cycles_self_refs
                .iter()
                .map(ToString::to_string)
                .collect(),
        ),
        (
            ReportLabel::Cycles,
            report
                .cycles_complex
                .iter()
                .map(|cycle| {
                    cycle
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" → ")
                })
                .collect(),
        ),
    ];
    for (label, items) in sections {
        output.push_str(&format!("[{}] ({})\n", t.report_label(label)?, items.len()));
        for item in items {
            output.push_str(&format!("  - {item}\n"));
        }
        output.push('\n');
    }
    for (label, count) in [
        (ReportLabel::SwapMatched, report.swap_matched),
        (ReportLabel::SwapNoMatch, report.swap_nomatch),
        (ReportLabel::SwapUncertain, report.swap_uncertain),
    ] {
        output.push_str(&format!("{}: {count}\n", t.report_label(label)?));
    }
    Ok(output)
}

pub fn render_issue(
    issue: &dtt_stellaris::game_data::DefinitionIssue,
    t: &Translator,
) -> Result<String, TranslationError> {
    use dtt_stellaris::game_data::DefinitionIssue as I;
    match issue {
        I::InvalidUtf8 => t.diagnostic_invalid_utf8(),
        I::InvalidSyntax => t.diagnostic_invalid_syntax(),
        I::ExpectedScalar => t.diagnostic_expected_scalar(),
        I::ExpectedObject => t.diagnostic_expected_object(),
        I::InvalidField { field } => t.diagnostic_invalid_field(field),
        I::Overwritten { previous } => t.diagnostic_overwritten(previous.as_deref().unwrap_or("—")),
        I::InlineCycle { script } => t.diagnostic_inline_cycle(script),
        I::InlineMissing { script } => t.diagnostic_inline_missing(script),
        I::InvalidInlineCall => t.diagnostic_invalid_inline_call(),
        I::IsolatedBranch => t.diagnostic_isolated_branch(),
        I::InvalidCondition => t.diagnostic_invalid_condition(),
        I::MalformedArgument => t.diagnostic_malformed_argument(),
        I::StructuredArgument => t.diagnostic_structured_argument(),
        I::DuplicateParameter => t.diagnostic_duplicate_parameter(),
        I::InvalidSwap { index, field } => t.diagnostic_invalid_swap(*index, field),
        I::DuplicateField { field } => t.diagnostic_duplicate_field(field),
        I::FieldCase { field, expected } => t.diagnostic_field_case(field, expected),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_context_is_independent_of_game_language_and_other_reports() {
        let report = GenerationReport {
            eligible: vec!["tech_测试".into()],
            ..Default::default()
        };
        let english = render_report(&report, AppLocale::En).unwrap();
        let chinese = render_report(&report, AppLocale::ZhHans).unwrap();
        assert!(english.contains("Eligible technologies"));
        assert!(chinese.contains("可用科技"));
        assert!(english.contains("tech_测试") && chinese.contains("tech_测试"));
        assert_eq!(english, render_report(&report, AppLocale::En).unwrap());
    }
}

fn render_condition(
    reason: &crate::UnknownConditionReason,
    t: &Translator,
) -> Result<String, TranslationError> {
    use crate::{ContextReason as C, UnknownConditionReason as R};
    use dtt_i18n::ConditionMessage as M;
    match reason {
        R::Trigger(_) => t.condition(M::Trigger),
        R::Operator { operator, .. } => Ok(format!(
            "{}: {}",
            t.condition(M::Operator)?,
            operator.symbol()
        )),
        R::MalformedArgument(_) => t.condition(M::MalformedArgument),
        R::StructuredArgument { keys, .. } => Ok(format!(
            "{}: {}",
            t.condition(M::StructuredArgument)?,
            keys.join(", ")
        )),
        R::Context {
            reason,
            scope_path,
            calls,
            ..
        } => {
            let label = t.condition(match reason {
                C::MissingPreviousScope => M::MissingPreviousScope,
                C::MissingEventSource => M::MissingEventSource,
                C::MissingFounderSpecies => M::MissingFounderSpecies,
                C::UnknownSpeciesRelation => M::UnknownSpeciesRelation,
                C::UnknownOwner => M::UnknownOwner,
                C::UnknownTrigger => M::UnknownTrigger,
                C::ExpectedScalar => M::ExpectedScalar,
                C::UnboundArgument => M::UnboundArgument,
                C::InvalidBoolean => M::InvalidBoolean,
                C::MissingIdentity => M::MissingIdentity,
                C::UnverifiedStructure => M::UnverifiedStructure,
                C::UnknownScope => M::UnknownScope,
                C::UnknownCollection => M::UnknownCollection,
                C::RecursiveScript => M::RecursiveScript,
                C::TypeMismatch { .. } => M::TypeMismatch,
            })?;
            let types = if let C::TypeMismatch { expected, actual } = reason {
                format!("{} → {expected}", actual.as_deref().unwrap_or("—"))
            } else {
                String::new()
            };
            Ok(format!(
                "{label} {types} [{}] [{}]",
                scope_path.join(" → "),
                calls.join(" → ")
            ))
        }
    }
}
