//! Tokenize and parse SQL the way every lint request does.

use crate::sql_lint::models::ParsedSql;

/// Parse `sql` under `dialect`, rejecting statements the parser kept as opaque text.
pub fn parse_sql(sql: &str, dialect: &str) -> Result<ParsedSql, String> {
    crate::sql_lint::_helpers::parsing::parse(sql, dialect)
}
