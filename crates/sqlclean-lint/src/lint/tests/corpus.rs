//! The lint corpus: each statement's findings on every dialect, against the ones accepted.

use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::lint::main::catalogue::catalogue;
use crate::lint::main::engine::lint_json;

const STATEMENTS: &str = "src/lint/tests/fixtures/lint_corpus.jsonl";
const ACCEPTED: &str = "src/lint/tests/fixtures/lint_corpus_accepted.tsv";
/// Set to rewrite the accepted findings from the current engine, which the commit's diff then shows.
const ACCEPT_VARIABLE: &str = "SQLCLEAN_ACCEPT_LINT_CORPUS";
const DIALECTS: [&str; 6] = [
    "snowflake",
    "duckdb",
    "postgres",
    "sqlite",
    "bigquery",
    "databricks",
];
const SHOWN_CHANGES: usize = 60;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// The lines whose findings differ from the accepted ones, as `-accepted` and `+found`; none after accepting.
pub(crate) fn corpus_changes() -> Result<Vec<String>, String> {
    let root: &str = env!("CARGO_MANIFEST_DIR");
    let statements: String = std::fs::read_to_string(format!("{root}/{STATEMENTS}"))
        .map_err(|error| error.to_string())?;
    let found: Vec<String> = findings(&statements)?;
    let path: String = format!("{root}/{ACCEPTED}");
    if std::env::var_os(ACCEPT_VARIABLE).is_some() {
        std::fs::write(&path, found.join("")).map_err(|error| error.to_string())?;
        return Ok(Vec::new());
    }
    let accepted: String = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let accepted: Vec<&str> = accepted.split_inclusive('\n').collect();
    let mut changes: Vec<String> = Vec::new();
    for index in 0..accepted.len().max(found.len()) {
        let (before, after) = (accepted.get(index).copied(), found.get(index).map(String::as_str));
        if before != after {
            changes.extend(before.map(|line| format!("-{line}")));
            changes.extend(after.map(|line| format!("+{line}")));
        }
    }
    Ok(changes)
}

/// Why the corpus failed: the first changed lines and how to accept them.
pub(crate) fn change_report(changes: &[String]) -> String {
    let mut report: String = format!(
        "{} lint corpus lines changed; read each one, then accept them with {ACCEPT_VARIABLE}=1 and commit the diff of {ACCEPTED}:\n",
        changes.len()
    );
    for line in changes.iter().take(SHOWN_CHANGES) {
        report.push_str(line);
    }
    report
}

fn findings(statements: &str) -> Result<Vec<String>, String> {
    let codes: Vec<&str> = catalogue().iter().map(|rule| rule.code).collect();
    let mut lines: Vec<String> = Vec::new();
    for entry in statements.lines() {
        let entry: Value = serde_json::from_str(entry).map_err(|error| error.to_string())?;
        let (id, sql) = (&entry["id"], entry["sql"].as_str().unwrap_or_default());
        for dialect in DIALECTS {
            let request: String =
                json!({"version": 1, "sql": sql, "dialect": dialect, "enabled_rules": codes})
                    .to_string();
            let summary: String = match lint_json(&request) {
                Ok(response) => summary(&response)?,
                Err(_) => "ERR".to_owned(),
            };
            lines.push(format!("{id}\t{dialect}\t{summary}\n"));
        }
    }
    Ok(lines)
}

/// Each finding as its Rule, range and a hash of everything it says, its fix included.
fn summary(response: &str) -> Result<String, String> {
    let response: Value = serde_json::from_str(response).map_err(|error| error.to_string())?;
    let mut written: String = String::new();
    for finding in response["diagnostics"].as_array().into_iter().flatten() {
        let code: &str = finding["code"].as_str().unwrap_or_default();
        let _ = write!(
            written,
            " {}@{}:{}#{:016x}",
            code.trim_start_matches("SQLCLEAN"),
            finding["start"],
            finding["end"],
            fnv(&finding.to_string())
        );
    }
    Ok(written.trim_start().to_owned())
}

fn fnv(text: &str) -> u64 {
    text.bytes()
        .fold(FNV_OFFSET, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME))
}
