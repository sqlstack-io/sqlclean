//! Whitespace-, comment- and keyword-case-insensitive SQL change fingerprints.

use sqlquack_fingerprint::FingerprintDialect;

use crate::sql_tokens::constants::FINGERPRINT_DIALECT_ALIASES;

/// Return SQLQuack's fingerprint of `sql` under `dialect_name`, the one SQLBuild stores; a dialect
/// without its own SQLQuack tokenizer is fingerprinted with the generic tokens.
pub fn query_fingerprint(sql: &str, dialect_name: &str) -> Result<String, String> {
    let name: &str = FINGERPRINT_DIALECT_ALIASES
        .iter()
        .find(|(alias, _)| *alias == dialect_name)
        .map_or(dialect_name, |(_, name)| name);
    let dialect = FingerprintDialect::from_name(name).unwrap_or(FingerprintDialect::Generic);
    sqlquack_fingerprint::query_fingerprint(sql, dialect).map_err(|error| error.to_string())
}
