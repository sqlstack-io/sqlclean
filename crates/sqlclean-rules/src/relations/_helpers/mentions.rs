//! The names that can refer to a relation.

use sqlquack_syntax::{Expression, ExpressionWalk};

/// Whether any node under `root` could refer to the relation named `name`.
pub(crate) fn mentions(root: &Expression, name: &str) -> bool {
    root.dfs().any(|node| node_mentions(node, name))
}

/// Whether `node` itself could refer to the relation named `name`.
pub(crate) fn node_mentions(node: &Expression, name: &str) -> bool {
    let same = |written: &str| written.eq_ignore_ascii_case(name);
    match node {
        Expression::Column(column) => match &column.table {
            Some(table) => same(&table.name),
            None => same(&column.name.name),
        },
        Expression::Star(star) => star.table.as_ref().is_some_and(|table| same(&table.name)),
        Expression::Identifier(identifier) => same(&identifier.name),
        Expression::Var(var) => same(&var.this),
        Expression::Table(table) => table
            .catalog
            .as_ref()
            .or(table.schema.as_ref())
            .is_some_and(|root| same(&root.name)),
        _ => false,
    }
}
