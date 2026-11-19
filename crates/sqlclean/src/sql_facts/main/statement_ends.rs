//! Where each statement of one SQL file ends, without collecting its facts.

/// The end of each statement of `sql`, after its terminator, parsed within a call-depth limit (None: the default); `None` when SQLQuack does not parse `dialect` yet.
pub fn statement_ends(
    sql: &str,
    dialect: &str,
    max_function_call_depth: Option<usize>,
) -> Result<Option<Vec<usize>>, String> {
    crate::sql_facts::_helpers::collection::statement_ends(sql, dialect, max_function_call_depth)
}
