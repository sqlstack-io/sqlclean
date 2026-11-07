use crate::sql_lint::types::LintRuleMetadata;

/// Every built-in lint Rule's code, message and remediation.
pub fn catalogue() -> Vec<LintRuleMetadata> {
    sqlquack_lint::catalogue()
}
