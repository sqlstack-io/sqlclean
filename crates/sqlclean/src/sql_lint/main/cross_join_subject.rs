//! The relation and CTE a `SQLCLEAN038` finding names, for relation-level Rule exceptions.

/// The `(relation, cte)` a cross join finding message names, `None` for the generic message.
#[must_use]
pub fn cross_join_subject(message: &str) -> Option<(String, Option<String>)> {
    sqlquack_lint::cross_join_subject(message)
}
