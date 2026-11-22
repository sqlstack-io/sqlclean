//! `principal_query`: the query a statement is or holds as its body.

use sqlquack_syntax::Expression;

use crate::trees::main::unwrapped::unwrapped;

/// The query `statement` is, or the body of the view, table, insert or prepared statement it defines.
pub(crate) fn principal_query(statement: &Expression) -> Option<&Expression> {
    let mut node: &Expression = statement;
    loop {
        node = match unwrapped(node) {
            query @ (Expression::Select(_)
            | Expression::Union(_)
            | Expression::Intersect(_)
            | Expression::Except(_)) => return Some(query),
            Expression::Subquery(subquery) if subquery.alias.is_none() => &subquery.this,
            Expression::CreateView(view) => &view.query,
            Expression::CreateTable(table) => table.as_select.as_ref()?,
            Expression::Insert(insert) => insert.query.as_ref()?,
            Expression::Prepare(prepare) => &prepare.statement,
            _ => return None,
        };
    }
}
