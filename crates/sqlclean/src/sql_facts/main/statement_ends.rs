//! Where each statement of one SQL file ends, without collecting its facts.

/// The end of each statement of `sql`, after its terminator; `None` when SQLQuack does not parse `dialect` yet.
pub fn statement_ends(sql: &str, dialect: &str) -> Result<Option<Vec<usize>>, String> {
    crate::sql_facts::_helpers::collection::statement_ends(sql, dialect)
}
