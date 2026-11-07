//! Tokenize and parse SQL the way every lint request does.

use crate::sql_lint::types::ParsedSql;

/// Parse `sql` under `dialect` with SQLQuack, rejecting statements the parser kept as opaque text.
pub fn parse_sql(sql: &str, dialect: &str) -> Result<ParsedSql, String> {
    sqlquack_lint::parse_sql(sql, dialect)
}
