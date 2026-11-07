//! Whitespace-, comment- and keyword-case-insensitive SQL change fingerprints.

use std::str::FromStr;

use polyglot_sql::{Dialect, DialectType};
use sha2::{Digest, Sha256};
use sqlquack_fingerprint::FingerprintDialect;

use crate::sql_tokens::constants::FINGERPRINT_DIALECT_ALIASES;
use crate::sql_tokens::main::canonical_tokens::canonical_tokens;
use crate::sql_tokens::main::token_texts::token_texts;

/// Return the fingerprint of `sql` under `dialect_name` that SQLBuild stores: SQLQuack's for the
/// dialects it fingerprints, and the Polyglot token stream's for the others, as SQLBuild does.
pub fn query_fingerprint(sql: &str, dialect_name: &str) -> Result<String, String> {
    let name: &str = FINGERPRINT_DIALECT_ALIASES
        .iter()
        .find(|(alias, _)| *alias == dialect_name)
        .map_or(dialect_name, |(_, name)| name);
    if let Ok(dialect) = FingerprintDialect::from_name(name) {
        return sqlquack_fingerprint::query_fingerprint(sql, dialect)
            .map_err(|error| error.to_string());
    }
    let dialect_type = DialectType::from_str(name)
        .map_err(|_| format!("unknown SQL fingerprint dialect '{dialect_name}'"))?;
    let dialect = Dialect::get(dialect_type);
    let tokens = dialect
        .tokenize(sql)
        .map_err(|error| format!("SQL tokenization failed: {error}"))?;
    let mut digest = Sha256::new();
    let spans = tokens.iter().map(|token| token.span.start..token.span.end);
    for token in canonical_tokens(token_texts(sql, spans)?, &dialect_type.to_string()) {
        digest.update(token.text.len().to_string().as_bytes());
        digest.update(b":");
        digest.update(token.text.as_bytes());
    }
    Ok(format!("{:x}", digest.finalize()))
}
