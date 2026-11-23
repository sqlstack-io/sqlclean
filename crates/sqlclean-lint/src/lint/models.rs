use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use sqlclean_rules::models::{MacroOutput, RuleInput};

use serde::{Deserialize, Serialize};
use sqlquack_syntax::Expression;
use sqlquack_syntax::tokens::models::Token;

/// A dialect lint reads, named by the parser that reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LintDialect {
    Snowflake,
    DuckDB,
    PostgreSQL,
    SQLite,
    BigQuery,
    Databricks,
}

/// SQL text tokenized and parsed under one dialect.
#[derive(Debug)]
pub struct ParsedSql {
    pub dialect: LintDialect,
    pub tokens: Vec<Token>,
    pub statements: Vec<Expression>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LintRequest {
    pub version: u32,
    pub sql: String,
    pub dialect: String,
    #[serde(default)]
    pub enabled_rules: Option<Vec<String>>,
    #[serde(default)]
    pub ignored_rules: Vec<String>,
    #[serde(default)]
    pub external_identifiers: Vec<String>,
    #[serde(default)]
    pub dependency_identifiers: Vec<String>,
    #[serde(default)]
    pub externally_referenced_ctes: Vec<String>,
    /// The harness the host wraps this body in; absent for an ordinary query.
    #[serde(default)]
    pub fixture_harness: Option<FixtureHarness>,
    #[serde(default)]
    pub allows_dynamic_output_star: bool,
    #[serde(default)]
    pub max_literal_length: Option<usize>,
    #[serde(default)]
    pub max_ranking_order_by: Option<usize>,
    /// Function calls nested deeper than this are a parse error; 256 when absent, at most 512.
    #[serde(default)]
    pub max_function_call_depth: Option<usize>,
    #[serde(default)]
    pub relation_keys: HashMap<String, Vec<Vec<String>>>,
    #[serde(default)]
    pub relation_names: HashMap<String, String>,
    /// The SQL each macro call in the query generated, so column qualification is judged as the macro wrote it.
    #[serde(default)]
    pub sql_macro_outputs: Vec<SqlMacroOutput>,
    /// The configuration file remediation text names; SQLClean's own when absent.
    #[serde(default)]
    pub config_file: Option<String>,
    /// The advice `SQLCLEAN044` gives before its limit setting; SQLClean's own when absent.
    #[serde(default)]
    pub long_literal_guidance: Option<String>,
}

/// The generated wrapper around a test or macro body: its terminal SELECT is ceremonial, and these names are generated.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FixtureHarness {
    /// Prefixes of the CTE names the harness generates, which are reachable even when unread.
    #[serde(default)]
    pub cte_prefixes: Vec<String>,
    /// Prefixes of the CTEs standing in for the body's inputs, which may read the empty relation with `SELECT *`.
    #[serde(default)]
    pub input_cte_prefixes: Vec<String>,
    /// The table function an input CTE reads, as `SELECT * FROM <name>()`, when it has no rows.
    #[serde(default)]
    pub empty_relation: Option<String>,
}

/// The character range one macro call's output fills, and the relations its arguments pass, which a plain parameter slot read expands to qualify with.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct SqlMacroOutput {
    pub start: usize,
    pub end: usize,
    #[serde(default)]
    pub relations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct LintResponse {
    pub version: u32,
    pub diagnostics: Vec<LintDiagnostic>,
    /// Why no Rule ran, for a dialect SQLQuack does not parse yet; absent when the SQL was linted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LintDiagnostic {
    pub code: &'static str,
    pub message: Cow<'static, str>,
    pub remediation: Cow<'static, str>,
    pub start: usize,
    pub end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<LintEdit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_unavailable_reason: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LintEdit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

#[derive(Debug, Clone, Copy)]
pub struct LintRuleMetadata {
    pub code: &'static str,
    pub message: &'static str,
    pub remediation: &'static str,
}

/// One lint request's Rules and what they read.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Report<'a> {
    pub(crate) request: &'a LintRequest,
    pub(crate) parsed: &'a ParsedSql,
    pub(crate) enabled: &'a HashSet<String>,
    pub(crate) input: &'a RuleInput<'a>,
}

/// The request's names as Rules compare them.
#[derive(Debug)]
pub(crate) struct RequestFacts<'a> {
    pub(crate) external: HashSet<String>,
    pub(crate) dependencies: HashSet<String>,
    pub(crate) read_by_host: HashSet<String>,
    pub(crate) macro_outputs: Vec<MacroOutput<'a>>,
}
