use std::fmt::Write as _;

use serde_json::{Map, Value, json};

use crate::lint::main::catalogue::catalogue;
use crate::lint::main::engine::lint_json;
use crate::lint::tests::test_types::CrossJoinTestCase;

/// The dependency sentinel the cross join cases name `order_policy`.
pub(crate) const POLICY_SENTINEL: &str = "__sqlclean_interpolation_7__";

pub(crate) fn nested_function_sql(depth: usize) -> String {
    format!("SELECT {}1{}", "F(".repeat(depth), ")".repeat(depth))
}

/// `depth` nested calls under an output alias.
pub(crate) fn aliased_calls_sql(depth: usize) -> String {
    format!(
        "SELECT {}1{} AS n FROM t",
        "F(".repeat(depth),
        ")".repeat(depth)
    )
}

/// `depth` nested calls compared in a WHERE clause.
pub(crate) fn compared_calls_sql(depth: usize) -> String {
    format!(
        "SELECT * FROM t WHERE {}1{} = 1",
        "F(".repeat(depth),
        ")".repeat(depth)
    )
}

/// `depth` nested calls under an alias inside a CTE.
pub(crate) fn cte_calls_sql(depth: usize) -> String {
    format!(
        "WITH c AS (SELECT {}1{} AS n FROM t) SELECT n FROM c",
        "F(".repeat(depth),
        ")".repeat(depth)
    )
}

/// `depth` nested calls around a subquery reading the fixture harness's empty relation.
pub(crate) fn calls_around_empty_relation_sql(depth: usize) -> String {
    format!(
        "SELECT {}(SELECT 1 FROM __empty_fixture()){}",
        "F(".repeat(depth),
        ")".repeat(depth)
    )
}

/// `depth` UNION ALL operands after a first SELECT, with no call.
pub(crate) fn union_chain_sql(depth: usize) -> String {
    format!("SELECT 1{}", " UNION ALL SELECT 1".repeat(depth))
}

pub(crate) fn nested_from_sql(depth: usize) -> String {
    format!(
        "SELECT * FROM {}orders{}",
        "(".repeat(depth),
        ")".repeat(depth)
    )
}

pub(crate) fn diagnostics(sql: &str) -> Result<Vec<Value>, String> {
    diagnostics_within(sql, None)
}

/// The Snowflake diagnostics of `sql`, with the request's `max_function_call_depth`, null when absent.
pub(crate) fn diagnostics_within(
    sql: &str,
    max_function_call_depth: Option<usize>,
) -> Result<Vec<Value>, String> {
    dialect_diagnostics_within(sql, "snowflake", max_function_call_depth)
}

/// The `dialect` diagnostics of `sql`, with the request's `max_function_call_depth`, null when absent.
pub(crate) fn dialect_diagnostics_within(
    sql: &str,
    dialect: &str,
    max_function_call_depth: Option<usize>,
) -> Result<Vec<Value>, String> {
    let request = json!({
        "version": 1,
        "sql": sql,
        "dialect": dialect,
        "max_function_call_depth": max_function_call_depth
    });
    let response = lint_json(&request.to_string())?;
    diagnostic_values(&response)
}

/// The `dialect` diagnostics of `sql` in the fixture harness, with the request's `max_function_call_depth`.
pub(crate) fn harness_diagnostics_within(
    sql: &str,
    dialect: &str,
    max_function_call_depth: Option<usize>,
) -> Result<Vec<Value>, String> {
    let request = json!({
        "version": 1,
        "sql": sql,
        "dialect": dialect,
        "max_function_call_depth": max_function_call_depth,
        "fixture_harness": fixture_harness()
    });
    let response = lint_json(&request.to_string())?;
    diagnostic_values(&response)
}

pub(crate) fn diagnostics_for_rules(sql: &str, rules: &[&str]) -> Result<Vec<Value>, String> {
    diagnostics_for_dialect(sql, "snowflake", rules)
}

pub(crate) fn diagnostics_for_dialect(
    sql: &str,
    dialect: &str,
    rules: &[&str],
) -> Result<Vec<Value>, String> {
    let request = json!({
        "version": 1,
        "sql": sql,
        "dialect": dialect,
        "enabled_rules": rules,
    });
    let response = lint_json(&request.to_string())?;
    diagnostic_values(&response)
}

fn diagnostic_values(response: &str) -> Result<Vec<Value>, String> {
    let payload: Value = serde_json::from_str(response).map_err(|error| error.to_string())?;
    payload["diagnostics"]
        .as_array()
        .cloned()
        .ok_or_else(|| "diagnostics should be an array".to_string())
}

/// The `SQLCLEAN038` messages for one cross join case.
pub(crate) fn cross_join_messages(test_case: &CrossJoinTestCase) -> Result<Vec<String>, String> {
    cross_join_texts(test_case, "message")
}

/// The `SQLCLEAN038` remediations for one cross join case.
pub(crate) fn cross_join_remediations(
    test_case: &CrossJoinTestCase,
) -> Result<Vec<String>, String> {
    cross_join_texts(test_case, "remediation")
}

fn cross_join_texts(test_case: &CrossJoinTestCase, field: &str) -> Result<Vec<String>, String> {
    let keys: Map<String, Value> = test_case
        .one_row_relations
        .iter()
        .map(|name| ((*name).to_owned(), json!([[]])))
        .collect();
    let request = json!({
        "version": 1,
        "sql": test_case.sql,
        "dialect": test_case.dialect,
        "enabled_rules": ["SQLCLEAN038"],
        "relation_keys": keys,
        "relation_names": { POLICY_SENTINEL: "order_policy" },
    });
    let response: Value = serde_json::from_str(&lint_json(&request.to_string())?)
        .map_err(|error| error.to_string())?;
    Ok(response["diagnostics"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|diagnostic| diagnostic[field].as_str().unwrap_or_default().to_owned())
        .collect())
}

/// A host fixture harness like a test runner's: generated CTE prefixes, input stand-ins and an empty relation.
pub(crate) fn fixture_harness() -> Value {
    json!({
        "cte_prefixes": ["__expected__", "__ref__", "__source__", "__macro_actual__", "__macro_expected__"],
        "input_cte_prefixes": ["__ref__", "__source__", "__seed__"],
        "empty_relation": "__empty_fixture",
    })
}

/// Each finding of `rule` on `sql`: its anchored text, and the SQL once its fix is applied.
pub(crate) fn pinned_findings(
    sql: &str,
    dialect: &str,
    rule: &str,
) -> Result<Vec<(String, Option<String>)>, String> {
    let found: Vec<Value> = diagnostics_for_dialect(sql, dialect, &[rule])
        .map_err(|error| error.chars().take(100).collect::<String>())?;
    Ok(found
        .iter()
        .map(|finding| (spanned_text(sql, finding), fixed_sql(sql, &finding["fix"])))
        .collect())
}

fn spanned_text(sql: &str, finding: &Value) -> String {
    let start: usize = finding["start"].as_u64().unwrap_or_default() as usize;
    let end: usize = finding["end"].as_u64().unwrap_or_default() as usize;
    sql.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn fixed_sql(sql: &str, fix: &Value) -> Option<String> {
    let start: usize = fix["start"].as_u64()? as usize;
    let end: usize = fix["end"].as_u64()? as usize;
    let head: String = sql.chars().take(start).collect();
    let tail: String = sql.chars().skip(end).collect();
    Some(format!("{head}{}{tail}", fix["replacement"].as_str()?))
}

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

/// The `-accepted` and `+found` lines of the corpus at crate paths `statements` and `accepted`.
pub(crate) fn corpus_changes(statements: &str, accepted: &str) -> Result<Vec<String>, String> {
    let root: &str = env!("CARGO_MANIFEST_DIR");
    let statements: String = std::fs::read_to_string(format!("{root}/{statements}"))
        .map_err(|error| error.to_string())?;
    let found: Vec<String> = corpus_findings(&statements)?;
    let path: String = format!("{root}/{accepted}");
    // Accepting writes the found lines first, so the comparison below then finds no change.
    std::env::var_os(ACCEPT_VARIABLE)
        .map(|_| std::fs::write(&path, found.join("")))
        .transpose()
        .map_err(|error| error.to_string())?;
    let accepted: String = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let accepted: Vec<&str> = accepted.split_inclusive('\n').collect();
    let changes: Vec<String> = (0..accepted.len().max(found.len()))
        .map(|index| {
            (
                accepted.get(index).copied(),
                found.get(index).map(String::as_str),
            )
        })
        .filter(|(before, after)| before != after)
        .flat_map(|(before, after)| {
            before
                .map(|line| format!("-{line}"))
                .into_iter()
                .chain(after.map(|line| format!("+{line}")))
        })
        .collect();
    Ok(changes)
}

/// Why the corpus failed: the first changed lines and how to accept them.
pub(crate) fn change_report(changes: &[String], accepted: &str) -> String {
    let mut report: String = format!(
        "{} lint corpus lines changed; read each one, then accept them with {ACCEPT_VARIABLE}=1 and commit the diff of {accepted}:\n",
        changes.len()
    );
    for line in changes.iter().take(SHOWN_CHANGES) {
        report.push_str(line);
    }
    report
}

fn corpus_findings(statements: &str) -> Result<Vec<String>, String> {
    let codes: Vec<&str> = catalogue().iter().map(|rule| rule.code).collect();
    let mut lines: Vec<String> = Vec::new();
    for entry in statements.lines() {
        let entry: Value = serde_json::from_str(entry).map_err(|error| error.to_string())?;
        let (id, sql) = (&entry["id"], entry["sql"].as_str().unwrap_or_default());
        for dialect in DIALECTS {
            let request: String =
                json!({"version": 1, "sql": sql, "dialect": dialect, "enabled_rules": codes})
                    .to_string();
            let summary: String = lint_json(&request).map_or_else(
                |_| Ok("ERR".to_owned()),
                |response| finding_summary(&response),
            )?;
            lines.push(format!("{id}\t{dialect}\t{summary}\n"));
        }
    }
    Ok(lines)
}

/// Each finding as its Rule, range and a hash of everything it says, its fix included.
fn finding_summary(response: &str) -> Result<String, String> {
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
            fnv_hash(&finding.to_string())
        );
    }
    Ok(written.trim_start().to_owned())
}

fn fnv_hash(text: &str) -> u64 {
    text.bytes().fold(FNV_OFFSET, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    })
}
