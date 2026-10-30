//! Typed, in-process lint of one SQL text.

use crate::sql_lint::models::{LintRequest, LintResponse};

/// Lint `request.sql` with the selected built-in Rules and return findings in source order.
pub fn lint(request: LintRequest) -> Result<LintResponse, String> {
    crate::sql_lint::_helpers::engine::lint(request)
}
