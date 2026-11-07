//! The lint request, response and parse types, which SQLQuack's lint engine defines.

use sqlquack_lint::lint::models as lint;

/// SQL text tokenized and parsed under one dialect by SQLQuack.
pub type ParsedSql = lint::ParsedSql;
/// One lint request: the SQL, its dialect, the selected Rules and what the host knows.
pub type LintRequest = lint::LintRequest;
/// The harness a host wraps a fixture body in.
pub type FixtureHarness = lint::FixtureHarness;
/// The SQL one macro call generated, located in the query.
pub type SqlMacroOutput = lint::SqlMacroOutput;
/// Findings in source order, or why lint is unavailable for the dialect.
pub type LintResponse = lint::LintResponse;
/// One finding, located by character offsets.
pub type LintDiagnostic = lint::LintDiagnostic;
/// One automatic fix, located by character offsets.
pub type LintEdit = lint::LintEdit;
/// A built-in Rule's code, message and remediation.
pub type LintRuleMetadata = lint::LintRuleMetadata;
