//! Tokenize and parse SQL the way every lint request does.

use crate::sql_lint::types::ParsedSql;

/// Parse `sql` under `dialect` within a call-depth limit (None: the default), rejecting statements the parser kept as opaque text.
pub fn parse_sql(
    sql: &str,
    dialect: &str,
    max_function_call_depth: Option<usize>,
) -> Result<ParsedSql, String> {
    sqlclean_lint::parse_sql(sql, dialect, max_function_call_depth)
}
