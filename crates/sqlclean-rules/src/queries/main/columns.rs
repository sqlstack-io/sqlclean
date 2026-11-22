//! `columns`: the columns under a node.

use sqlquack_syntax::expressions::names::models::Column;
use sqlquack_syntax::{Expression, ExpressionWalk};

/// Every column under `expression`.
pub(crate) fn columns(expression: &Expression) -> Vec<&Column> {
    expression
        .dfs()
        .filter_map(|node| match node {
            Expression::Column(column) => Some(column.as_ref()),
            _ => None,
        })
        .collect()
}
