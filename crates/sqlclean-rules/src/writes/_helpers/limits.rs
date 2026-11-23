//! What limits the rows a write changes.

use sqlquack_syntax::expressions::queries::models::Join;
use sqlquack_syntax::expressions::statements::models::{Delete, Update};
use sqlquack_syntax::{Expression, ExpressionWalk};

/// Whether a WHERE, LIMIT or join on the target limits an UPDATE; a join inside FROM cannot read it.
pub(crate) fn update_is_limited(update: &Update) -> bool {
    update
        .where_clause
        .as_ref()
        .is_some_and(|clause| reads_rows(&clause.this))
        || update.limit.is_some()
        || update.table_joins.iter().any(join_is_limiting)
}

pub(crate) fn delete_is_limited(delete: &Delete) -> bool {
    delete
        .where_clause
        .as_ref()
        .is_some_and(|clause| reads_rows(&clause.this))
        || delete.limit.is_some()
        || delete.joins.iter().any(join_is_limiting)
}

/// Whether a predicate depends on the rows: it reads a column or a subquery, unlike `WHERE TRUE`.
fn reads_rows(predicate: &Expression) -> bool {
    predicate.dfs().any(|node| {
        matches!(
            node,
            Expression::Column(_) | Expression::Select(_) | Expression::Subquery(_)
        )
    })
}

fn join_is_limiting(join: &Join) -> bool {
    join.on.as_ref().is_some_and(reads_rows) || !join.using.is_empty()
}
