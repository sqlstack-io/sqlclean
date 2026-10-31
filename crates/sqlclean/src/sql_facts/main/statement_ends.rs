//! Where each statement of one SQL file ends, without collecting its facts.

/// Parse `sql` under `dialect` and return the end of each statement, after its terminator.
pub fn statement_ends(sql: &str, dialect: &str) -> Result<Vec<usize>, String> {
    crate::sql_facts::_helpers::collection::statement_ends(sql, dialect)
}
