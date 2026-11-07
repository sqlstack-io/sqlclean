//! Check a dialect name against the dialects lint knows.

/// `Ok` when `name` names a known SQL dialect, such as `duckdb` or `snowflake`.
pub fn validate_dialect(name: &str) -> Result<(), String> {
    sqlquack_lint::validate_dialect(name)
}
