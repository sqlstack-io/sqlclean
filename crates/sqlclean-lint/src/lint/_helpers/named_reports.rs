//! The diagnostics whose words depend on what their Rule found.

use std::borrow::Cow;

use crate::dialects::main::profile::profile;
use crate::lint::_helpers::cross_join_words::cross_join_findings;
use crate::lint::_helpers::engine::UNSTABLE_ROW_NUMBER;
use crate::lint::_helpers::rule_edits::rendered;
use crate::lint::_helpers::text_views::literals;
use crate::lint::models::{LintDiagnostic, LintEdit, LintRuleMetadata, Report};
use crate::quality::constants as quality;
use crate::quality::main::setting_help::setting_help;
use crate::quality::models::Setting;
use sqlclean_rules::models::OutputKind;
use sqlquack_syntax::expressions::names::models::Identifier;
use sqlquack_syntax::tokens::models::Span;

const PRIORITY_FIX_REFUSAL: &str =
    "Choosing a priority column and tie-breaker requires author intent";
const KEY_FIX_REFUSAL: &str = "Declaring a key or adding a tie-breaker requires author intent";
const UNORDERED_FIX_REFUSAL: &str = "ordering columns and a stable tie-breaker require user intent";
const PARSE_ONCE_FIX_REFUSAL: &str =
    "Parsing the payload once requires naming a column in the CTE that reads it";
const CONSTANT_FIX_REFUSAL: &str =
    "Moving the value to a constant requires choosing its name and placement";
const COMMENT_FIX_REFUSAL: &str = "Comments inside the removed column would be lost";

/// A Rule's metadata and the words of one finding.
type Worded = (
    &'static LintRuleMetadata,
    Cow<'static, str>,
    Cow<'static, str>,
);

pub(super) fn named_findings(report: &Report<'_>) -> Vec<LintDiagnostic> {
    let mut found: Vec<LintDiagnostic> = cross_join_findings(report);
    found.extend(ranking_findings(report));
    found.extend(output_findings(report));
    found.extend(parse_findings(report));
    found.extend(literal_findings(report));
    found
}

fn ranking_findings(report: &Report<'_>) -> Vec<LintDiagnostic> {
    let cap: bool = report.enabled.contains(quality::RANKING_SORT_CAP.code);
    let proof: bool = report.enabled.contains(UNSTABLE_ROW_NUMBER.code);
    if !(cap || proof) {
        return Vec::new();
    }
    let limit: usize = report.input.settings.max_ranking_order_by;
    let mut found: Vec<LintDiagnostic> = Vec::new();
    for ranking in sqlclean_rules::ranking_orders(report.input) {
        if cap && ranking.exceeds_cap {
            let help: String = setting_help(&Setting {
                purpose: &format!(
                    "To allow {} sort keys (the current value is {limit})",
                    ranking.sort_keys
                ),
                config_file: config_file(report),
                section: quality::RULES_THRESHOLDS_SECTION,
                key: quality::MAX_RANKING_ORDER_BY_KEY,
                value: &ranking.sort_keys.to_string(),
            });
            let remediation = format!("{} {help}", quality::RANKING_SORT_GUIDANCE);
            found.push(worded(
                ranking.span,
                PRIORITY_FIX_REFUSAL,
                (
                    &quality::RANKING_SORT_CAP,
                    Cow::Borrowed(quality::RANKING_SORT_CAP.message),
                    Cow::Owned(remediation),
                ),
            ));
        }
        if proof && ranking.unordered {
            found.push(worded(
                ranking.span,
                UNORDERED_FIX_REFUSAL,
                (
                    &UNSTABLE_ROW_NUMBER,
                    Cow::Borrowed(UNSTABLE_ROW_NUMBER.message),
                    Cow::Borrowed(UNSTABLE_ROW_NUMBER.remediation),
                ),
            ));
        }
        if proof && ranking.unproven {
            found.push(worded(
                ranking.span,
                KEY_FIX_REFUSAL,
                (
                    &UNSTABLE_ROW_NUMBER,
                    Cow::Borrowed(quality::UNPROVEN_RANKING_MESSAGE),
                    Cow::Borrowed(quality::UNPROVEN_RANKING_REMEDIATION),
                ),
            ));
        }
    }
    found
}

fn output_findings(report: &Report<'_>) -> Vec<LintDiagnostic> {
    let rule: &LintRuleMetadata = &quality::UNUSED_CTE_OUTPUT;
    if !report.enabled.contains(rule.code) {
        return Vec::new();
    }
    sqlclean_rules::unused_cte_outputs(report.input)
        .into_iter()
        .map(|output| {
            let fix: Option<LintEdit> = output
                .finding
                .fix
                .as_ref()
                .and_then(|edit| rendered(edit, &report.request.sql, &report.parsed.tokens));
            let remediation: &'static str = match output.kind {
                OutputKind::Plain => rule.remediation,
                OutputKind::Distinct => quality::DISTINCT_UNUSED_REMEDIATION,
                OutputKind::GroupByAll => quality::GROUP_BY_ALL_UNUSED_REMEDIATION,
                OutputKind::GroupKey => quality::GROUP_KEY_UNUSED_REMEDIATION,
            };
            LintDiagnostic {
                code: rule.code,
                message: Cow::Borrowed(rule.message),
                remediation: Cow::Borrowed(remediation),
                start: output.finding.span.start,
                end: output.finding.span.end,
                fix_unavailable_reason: fix
                    .is_none()
                    .then_some(output.refusal.unwrap_or(COMMENT_FIX_REFUSAL)),
                fix,
            }
        })
        .collect()
}

fn parse_findings(report: &Report<'_>) -> Vec<LintDiagnostic> {
    if !report.enabled.contains(quality::REPEATED_JSON_PARSE.code) {
        return Vec::new();
    }
    let quote: char = profile(report.parsed.dialect).identifier_quote;
    sqlclean_rules::repeated_json_parses(report.input)
        .into_iter()
        .map(|parse| {
            let name: String = written(parse.column, quote);
            let column: String = match parse.qualifier {
                Some(table) => format!("{}.{name}", written(table, quote)),
                None => name.clone(),
            };
            let (function, count) = (parse.function, parse.count);
            let remediation = format!(
                "{function}({column}) is parsed {count} times in one SELECT. Parse it once in the CTE that reads it, for example `{function}({name}) AS payload`, and read every field from that column in a later step."
            );
            worded(parse.span, PARSE_ONCE_FIX_REFUSAL, (
                &quality::REPEATED_JSON_PARSE,
                Cow::Borrowed(quality::REPEATED_JSON_PARSE.message),
                Cow::Owned(remediation)))
        })
        .collect()
}

/// The identifier as authored, re-quoted with the dialect's identifier quote when it was quoted.
fn written(identifier: &Identifier, quote: char) -> String {
    if identifier.quoted {
        let escaped: String = identifier.name.replace(quote, &format!("{quote}{quote}"));
        format!("{quote}{escaped}{quote}")
    } else {
        identifier.name.clone()
    }
}

fn literal_findings(report: &Report<'_>) -> Vec<LintDiagnostic> {
    if !report.enabled.contains(quality::LONG_LITERAL.code) {
        return Vec::new();
    }
    let limit: usize = report
        .request
        .max_literal_length
        .unwrap_or(quality::DEFAULT_MAX_LITERAL_LENGTH);
    let guidance: &str = report
        .request
        .long_literal_guidance
        .as_deref()
        .unwrap_or(quality::LONG_LITERAL_GUIDANCE);
    sqlclean_rules::long_literals(&literals(&report.parsed.tokens), limit)
        .into_iter()
        .map(|literal| {
            let length: usize = literal.length;
            let help: String = setting_help(&Setting {
                purpose: &format!(
                    "To allow literals of {length} characters (the current value is {limit})"
                ),
                config_file: config_file(report),
                section: quality::RULES_THRESHOLDS_SECTION,
                key: quality::MAX_LITERAL_LENGTH_KEY,
                value: &length.to_string(),
            });
            worded(
                literal.span,
                CONSTANT_FIX_REFUSAL,
                (
                    &quality::LONG_LITERAL,
                    Cow::Borrowed(quality::LONG_LITERAL.message),
                    Cow::Owned(format!("{guidance} {help}")),
                ),
            )
        })
        .collect()
}

fn config_file<'a>(report: &Report<'a>) -> &'a str {
    report
        .request
        .config_file
        .as_deref()
        .unwrap_or(crate::constants::DEFAULT_CONFIG_FILE)
}

fn worded(
    span: Span,
    refusal: &'static str,
    (rule, message, remediation): Worded,
) -> LintDiagnostic {
    LintDiagnostic {
        code: rule.code,
        message,
        remediation,
        start: span.start,
        end: span.end,
        fix: None,
        fix_unavailable_reason: Some(refusal),
    }
}
