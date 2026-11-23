//! `principal_query`: the query a statement is or holds as its body.

use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::trees::main::unwrapped::unwrapped;

/// The query `statement` is, or the body of the statement that defines, stores, explains or copies it.
pub(crate) fn principal_query(statement: &Expression) -> Option<&Expression> {
    let mut node: &Expression = statement;
    loop {
        node = match unwrapped(node) {
            query @ (Expression::Select(_)
            | Expression::Union(_)
            | Expression::Intersect(_)
            | Expression::Except(_)
            | Expression::Values(_)) => return Some(query),
            Expression::Subquery(subquery) if subquery.alias.is_none() => &subquery.this,
            Expression::CreateView(view) => &view.query,
            Expression::CreateTable(table) => table.as_select.as_ref()?,
            Expression::Insert(insert) => insert.query.as_ref()?,
            Expression::Prepare(prepare) => &prepare.statement,
            Expression::Describe(described) => &described.target,
            Expression::Copy(copy) => &copy.this,
            Expression::Cache(cache) => cache.query.as_ref()?,
            Expression::PipeOperator(pipe) => &pipe.this,
            Expression::Summarize(summary) => &summary.this,
            Expression::Property(property) => property.value.as_deref()?,
            Expression::UtilityStatement(utility) => utility.operands.last()?,
            Expression::CreateObject(object) => return defined_query(&object.operands),
            altered @ Expression::AlterView(_) => altered.children().into_iter().find(is_query)?,
            _ => return None,
        };
    }
}

/// The query an object's clauses write as its action, never one inside their expressions.
fn defined_query(operands: &[Expression]) -> Option<&Expression> {
    let mut pending: Vec<&Expression> = operands.iter().collect();
    while let Some(node) = pending.pop() {
        match node {
            Expression::Tuple(clause) => pending.extend(&clause.expressions),
            other if is_query(&other) => return Some(other),
            _ => {}
        }
    }
    None
}

fn is_query(node: &&Expression) -> bool {
    matches!(
        node,
        Expression::Select(_)
            | Expression::Union(_)
            | Expression::Intersect(_)
            | Expression::Except(_)
    )
}
