//! The built-in Rules on one file, through the engine.

use std::collections::BTreeSet;

use sqlclean::sql_lint::constants::LINT_API_VERSION;
use sqlclean::sql_lint::main::fill_finding_path::fill_finding_path;
use sqlclean::sql_lint::main::lint::lint;
use sqlclean::sql_lint::models::{LintDiagnostic, LintRequest};

use crate::checking::constants::{
    PARSE_ERROR_CODE, PARSE_ERROR_MESSAGE, PARSE_ERROR_REMEDIATION, UNSAFE_FIX_BUILTIN_RULES,
};
use crate::checking::models::{Finding, FixSafety};
use crate::config::constants::CONFIG_FILE_NAME;
use crate::config::models::Thresholds;
use crate::discovery::models::SourceFile;
use crate::positions::models::Edit;

/// The built-in findings of `file`, or one parse-error finding when the engine rejects it.
pub(crate) fn builtin_findings(
    file: &SourceFile,
    codes: &BTreeSet<String>,
    thresholds: Thresholds,
) -> Vec<Finding> {
    let request = LintRequest {
        version: LINT_API_VERSION,
        sql: file.text.clone(),
        dialect: file.dialect.clone(),
        enabled_rules: Some(codes.iter().cloned().collect()),
        max_literal_length: Some(thresholds.max_literal_length),
        max_ranking_order_by: Some(thresholds.max_ranking_order_by),
        config_file: Some(CONFIG_FILE_NAME.to_owned()),
        ..LintRequest::default()
    };
    match lint(request) {
        Ok(response) => response
            .diagnostics
            .into_iter()
            .map(|diagnostic| finding(file, diagnostic))
            .collect(),
        Err(error) => vec![parse_error(file, &error)],
    }
}

/// The finding for a file the parser rejects.
pub(crate) fn parse_error(file: &SourceFile, error: &str) -> Finding {
    Finding {
        code: PARSE_ERROR_CODE.to_owned(),
        path: file.path.clone(),
        start: 0,
        end: 0,
        message: format!("{PARSE_ERROR_MESSAGE} as {}: {error}", file.dialect),
        remediation: PARSE_ERROR_REMEDIATION.to_owned(),
        edits: Vec::new(),
        fix: None,
    }
}

fn finding(file: &SourceFile, diagnostic: LintDiagnostic) -> Finding {
    let edits: Vec<Edit> = diagnostic
        .fix
        .into_iter()
        .map(|edit| Edit {
            start: edit.start,
            end: edit.end,
            text: edit.replacement,
        })
        .collect();
    Finding {
        code: diagnostic.code.to_owned(),
        path: file.path.clone(),
        start: diagnostic.start,
        end: diagnostic.end,
        message: diagnostic.message.into_owned(),
        remediation: fill_finding_path(&diagnostic.remediation, &file.path),
        fix: (!edits.is_empty()).then(|| builtin_fix_safety(diagnostic.code)),
        edits,
    }
}

/// How safe the fixes of the built-in Rule `code` are.
pub(crate) fn builtin_fix_safety(code: &str) -> FixSafety {
    if UNSAFE_FIX_BUILTIN_RULES.contains(&code) {
        FixSafety::Unsafe
    } else {
        FixSafety::Safe
    }
}
