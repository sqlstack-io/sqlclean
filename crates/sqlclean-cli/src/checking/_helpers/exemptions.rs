//! Configured path ignores and reviewed exceptions, and exceptions that no longer match.

use std::collections::{BTreeSet, HashSet};

use globset::GlobSet;
use sqlclean::sql_lint::main::cross_join_subject::cross_join_subject;

use crate::checking::constants::{CROSS_JOIN_CODE, PARSE_ERROR_CODE};
use crate::checking::models::{Finding, StaleException};
use crate::config::models::{RuleException, RulesConfig};
use crate::discovery::main::path_patterns::path_patterns;

/// Findings left after ignores and exceptions, and the exceptions this check proves stale.
pub(crate) fn apply_exemptions(
    findings: Vec<Finding>,
    rules: &RulesConfig,
    checked_paths: &HashSet<String>,
    selected: &BTreeSet<String>,
) -> Result<(Vec<Finding>, Vec<StaleException>), String> {
    let mut ignores: Vec<(GlobSet, &[String])> = Vec::new();
    for ignore in &rules.rule_ignores {
        ignores.push((path_patterns(&ignore.paths)?, ignore.rules.as_slice()));
    }
    let mut used: HashSet<usize> = HashSet::new();
    let kept: Vec<Finding> = findings
        .into_iter()
        .filter(|finding| finding.code == PARSE_ERROR_CODE || !is_ignored(finding, &ignores))
        .filter(
            |finding| match exception_for(finding, &rules.rule_exceptions) {
                Some(index) => {
                    used.insert(index);
                    false
                }
                None => true,
            },
        )
        .collect();
    let stale: Vec<StaleException> = rules
        .rule_exceptions
        .iter()
        .enumerate()
        .filter(|(index, exception)| {
            !used.contains(index)
                && checked_paths.contains(&exception.path)
                && selected.contains(&exception.rule)
        })
        .map(|(_, exception)| StaleException {
            rule: exception.rule.clone(),
            path: exception.path.clone(),
            relation: exception.relation.clone(),
        })
        .collect();
    Ok((kept, stale))
}

fn is_ignored(finding: &Finding, ignores: &[(GlobSet, &[String])]) -> bool {
    ignores
        .iter()
        .any(|(paths, codes)| paths.is_match(&finding.path) && selects(codes, &finding.code))
}

fn selects(selectors: &[String], code: &str) -> bool {
    selectors
        .iter()
        .any(|selector| code.starts_with(selector.as_str()))
}

/// The index of the exception that covers `finding`; parse errors are never excepted.
fn exception_for(finding: &Finding, exceptions: &[RuleException]) -> Option<usize> {
    if finding.code == PARSE_ERROR_CODE {
        return None;
    }
    exceptions
        .iter()
        .position(|exception| covers(exception, finding))
}

fn covers(exception: &RuleException, finding: &Finding) -> bool {
    if exception.rule != finding.code || exception.path != finding.path {
        return false;
    }
    let Some(relation) = &exception.relation else {
        return true;
    };
    if finding.code != CROSS_JOIN_CODE {
        return false;
    }
    let Some((joined, cte)) = cross_join_subject(&finding.message) else {
        return false;
    };
    let cte_matches: bool = match (&exception.cte, cte) {
        (None, _) => true,
        (Some(expected), Some(cte)) => cte.eq_ignore_ascii_case(expected),
        (Some(_), None) => false,
    };
    joined.eq_ignore_ascii_case(relation) && cte_matches
}
