import i18n from "./index";
import type { AppError, GenerationDiagnosticItemDto } from "@/ipc/bindings";
import { currentLocale } from "./controller";
import { listFormatter } from "./format";
export function errorTitle(error: AppError) { return i18n.t($ => $.titles[error.code], { ns: "errors" }); }
export function diagnosticMessage(item: GenerationDiagnosticItemDto): { summary: string; detail?: string } {
  const t = i18n.getFixedT(null, "diagnostics");
  const list = (items: string[]) => listFormatter(currentLocale()).format(items);
  switch (item.kind) {
    case "missing_mod_descriptor": return { summary: t($ => $.missing_mod_descriptor, { name: item.modName }) };
    case "unknown_condition": return { summary: t($ => $.unknown_condition, { name: item.name, count: item.occurrences, items: list(item.technologies) }), detail: conditionMessage(item.reason) };
    case "deferred_condition": return { summary: t($ => $.deferred_condition, { name: item.name, count: item.occurrences, items: list(item.technologies) }) };
    case "game_data": return { summary: t($ => $.game_data, { source: item.source, subject: item.subject ?? "—", category: item.category, reason: issueMessage(item.issue), offset: item.byteOffset ?? "—" }), ...(item.technicalDetail ? { detail: item.technicalDetail } : {}) };
    case "unhandled_definition": return { summary: t($ => $.unhandled_definition, { technology: item.technology, source: item.source, items: list(item.fields) }) };
    case "localisation": return { summary: t($ => $.localisation, { ...item, reason: t($ => $.localisationReasons[item.reason]) }), detail: item.technicalDetail };
    case "self_reference": return { summary: t($ => $.self_reference, item) };
    case "cycle": return { summary: t($ => $.cycle, { items: item.technologies.join(" → ") }) };
    case "write_failed": return { summary: t($ => $.write_failed, { path: item.path, operation: t($ => $.operations[item.operation]) }), detail: item.technicalDetail };
  }
}

function issueMessage(issue: import("@/ipc/bindings/DefinitionIssueDto").DefinitionIssueDto): string {
  const t = i18n.getFixedT(null, "diagnostics");
  switch (issue.kind) {
    case "invalid_utf8": return t($ => $.issues.invalid_utf8);
    case "invalid_syntax": return t($ => $.issues.invalid_syntax);
    case "expected_scalar": return t($ => $.issues.expected_scalar);
    case "expected_object": return t($ => $.issues.expected_object);
    case "invalid_field": return t($ => $.issues.invalid_field, { field: issue.field });
    case "overwritten": return t($ => $.issues.overwritten, { previous: issue.previous ?? "—" });
    case "inline_cycle": return t($ => $.issues.inline_cycle, { script: issue.script });
    case "inline_missing": return t($ => $.issues.inline_missing, { script: issue.script });
    case "invalid_inline_call": return t($ => $.issues.invalid_inline_call);
    case "isolated_branch": return t($ => $.issues.isolated_branch);
    case "invalid_condition": return t($ => $.issues.invalid_condition);
    case "malformed_argument": return t($ => $.issues.malformed_argument);
    case "structured_argument": return t($ => $.issues.structured_argument);
    case "duplicate_parameter": return t($ => $.issues.duplicate_parameter);
    case "invalid_swap": return t($ => $.issues.invalid_swap, { index: issue.index, field: issue.field });
    case "duplicate_field": return t($ => $.issues.duplicate_field, { field: issue.field });
    case "field_case": return t($ => $.issues.field_case, { field: issue.field, expected: issue.expected });
  }
}

function conditionMessage(reason: import("@/ipc/bindings/UnknownConditionDto").UnknownConditionDto): string {
  const t = i18n.getFixedT(null, "diagnostics");
  switch (reason.kind) {
    case "trigger": return t($ => $.conditions.trigger);
    case "operator": return `${t($ => $.conditions.operator)}: ${reason.operator}`;
    case "malformed_argument": return t($ => $.conditions.malformed_argument);
    case "structured_argument": return `${t($ => $.conditions.structured_argument)}: ${reason.keys.join(", ")}`;
    case "context": return `${t($ => $.conditions[reason.reason])} ${reason.actual ?? ""} → ${reason.expected ?? ""} [${reason.scopePath.join(" → ")}] [${reason.calls.join(" → ")}]`;
  }
}
