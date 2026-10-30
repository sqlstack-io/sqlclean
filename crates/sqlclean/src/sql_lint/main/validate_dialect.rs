//! Check a dialect name against the dialects the parser supports.

use std::str::FromStr;

use polyglot_sql::DialectType;

/// `Ok` when `name` names a supported SQL dialect, such as `duckdb` or `snowflake`.
pub fn validate_dialect(name: &str) -> Result<(), String> {
    match DialectType::from_str(name) {
        Ok(_) => Ok(()),
        Err(error) => Err(format!("unknown SQL dialect '{name}': {error}")),
    }
}
