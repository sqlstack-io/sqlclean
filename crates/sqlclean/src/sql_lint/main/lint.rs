//! Typed, in-process lint of one SQL text.

use crate::sql_lint::types::{LintRequest, LintResponse};

/// Lint `request.sql` with the selected built-in Rules and return findings in source order.
pub fn lint(request: LintRequest) -> Result<LintResponse, String> {
    sqlquack_lint::lint(request)
}
