//! `node_joins`: the joins a node owns, a write's included.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::Join;

/// The joins written directly on `node`.
pub(crate) fn node_joins(node: &Expression) -> Vec<&Join> {
    match node {
        Expression::Update(update) => update
            .table_joins
            .iter()
            .chain(&update.from_joins)
            .collect(),
        Expression::Delete(delete) => delete.joins.iter().collect(),
        _ => node.own_joins().iter().collect(),
    }
}
