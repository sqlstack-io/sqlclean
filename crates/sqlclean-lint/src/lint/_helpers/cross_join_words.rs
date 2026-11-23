//! SQLCLEAN038's words, with the relation's name as written.

use std::borrow::Cow;
use std::collections::HashMap;

use sqlclean_rules::models::JoinedName;
use sqlquack_syntax::expressions::names::models::Identifier;

use crate::constants::SETTING_SNIPPET_INDENT;
use crate::lint::_helpers::engine::CROSS_JOIN;
use crate::lint::_helpers::rule_edits::char_slice;
use crate::lint::constants::{
    CROSS_JOIN_CTE_MARKER, CROSS_JOIN_PATH_PLACEHOLDER, CROSS_JOIN_PREFIX,
    CROSS_JOIN_REASON_PLACEHOLDER, CROSS_JOIN_SUFFIX, CROSS_JOIN_UNNAMED_MESSAGE,
};
use crate::lint::models::{LintDiagnostic, Report};

const CROSS_JOIN_FIX_REASON: &str = "a keyed relationship requires user intent";

pub(super) fn cross_join_findings(report: &Report<'_>) -> Vec<LintDiagnostic> {
    if !report.enabled.contains(CROSS_JOIN.code) {
        return Vec::new();
    }
    let sentinels: HashMap<String, &str> = report
        .request
        .relation_names
        .iter()
        .map(|(sentinel, name)| (sentinel.to_ascii_lowercase(), name.as_str()))
        .collect();
    sqlclean_rules::cross_joins(report.input)
        .into_iter()
        .map(|finding| {
            let relation: Option<String> =
                finding_relation(&report.request.sql, finding.relation.as_ref(), &sentinels);
            LintDiagnostic {
                code: CROSS_JOIN.code,
                message: match &relation {
                    Some(relation) => Cow::Owned(cross_join_message(relation, finding.cte)),
                    None => Cow::Borrowed(CROSS_JOIN_UNNAMED_MESSAGE),
                },
                remediation: match relation {
                    Some(_) => Cow::Borrowed(CROSS_JOIN.remediation),
                    None => Cow::Owned(file_wide_remediation(CROSS_JOIN.code)),
                },
                start: finding.span.start,
                end: finding.span.end,
                fix_unavailable_reason: Some(CROSS_JOIN_FIX_REASON),
                fix: None,
            }
        })
        .collect()
}

/// The written name of a cross join's relation, when the finding names one.
fn finding_relation(
    sql: &str,
    relation: Option<&JoinedName<'_>>,
    sentinels: &HashMap<String, &str>,
) -> Option<String> {
    let name: &JoinedName<'_> = relation?;
    Some(relation_name(sql, name, sentinels))
}

fn relation_name(sql: &str, name: &JoinedName<'_>, sentinels: &HashMap<String, &str>) -> String {
    match name {
        JoinedName::Read(read) => (*read).to_owned(),
        JoinedName::Alias(alias) => authored(sql, alias).to_owned(),
        JoinedName::Table(table) => sentinels.get(&table.name.to_ascii_lowercase()).map_or_else(
            || authored(sql, table).to_owned(),
            |known| (*known).to_owned(),
        ),
        JoinedName::Qualified(parts) => parts
            .iter()
            .map(|part| part.map_or("", |part| authored(sql, part)))
            .collect::<Vec<&str>>()
            .join("."),
    }
}

/// An unquoted name as authored, though PostgreSQL's parser folds it to lower case.
fn authored<'a>(sql: &'a str, identifier: &'a Identifier) -> &'a str {
    identifier
        .span
        .filter(|_| !identifier.quoted)
        .and_then(|span| char_slice(sql, span.start, span.end))
        .filter(|text| text.eq_ignore_ascii_case(&identifier.name))
        .unwrap_or(identifier.name.as_str())
}

/// Help for a join no relation names: fix it, or record the file-wide exception it allows.
fn file_wide_remediation(code: &str) -> String {
    let entry: String = [
        "[[rules.rule_exceptions]]".to_owned(),
        format!("rule = \"{code}\""),
        format!("path = \"{CROSS_JOIN_PATH_PLACEHOLDER}\""),
        format!("reason = \"{CROSS_JOIN_REASON_PLACEHOLDER}\""),
    ]
    .iter()
    .map(|line| format!("\n{SETTING_SNIPPET_INDENT}{line}"))
    .collect();
    format!(
        "Use a keyed JOIN, or make one side provably at most one row. This join reads no named \
         relation, so a relation-level exception cannot cover it; when the cartesian product is \
         intentional, record a reviewed file-wide exception in the configuration file:{entry}"
    )
}

/// The finding message naming the joined relation and, inside a CTE, that CTE.
fn cross_join_message(relation: &str, cte: Option<&str>) -> String {
    match cte {
        Some(cte) => {
            format!("{CROSS_JOIN_PREFIX}{relation}{CROSS_JOIN_CTE_MARKER}{cte}{CROSS_JOIN_SUFFIX}")
        }
        None => format!("{CROSS_JOIN_PREFIX}{relation}{CROSS_JOIN_SUFFIX}"),
    }
}
