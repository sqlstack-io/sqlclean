//! The joins a FROM clause writes.

use sqlquack_syntax::expressions::queries::models::{JoinKind, Select};
use sqlquack_syntax::{Expression, ExpressionWalk};

/// Whether the FROM clause writes a join, which binds tighter than its commas.
pub(crate) fn has_explicit_join(select: &Select) -> bool {
    !select.joins.is_empty()
        || select
            .from
            .iter()
            .flat_map(|from| &from.expressions)
            .any(writes_a_join)
}

/// Whether a join is written anywhere in FROM item `item`.
fn writes_a_join(item: &Expression) -> bool {
    item.dfs()
        .flat_map(Expression::own_joins)
        .any(|join| join.kind != JoinKind::Implicit)
}
