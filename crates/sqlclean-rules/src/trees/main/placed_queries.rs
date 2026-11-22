//! `placed_queries`: the queries written as steps of a statement rather than inside an expression.

use sqlquack_syntax::Expression;

use crate::trees::main::principal_query::principal_query;
use crate::trees::main::unwrapped::unwrapped;

/// The statement's body, and each CTE body and derived table reached from it without entering an expression.
pub(crate) fn placed_queries(statement: &Expression) -> Vec<&Expression> {
    let mut pending: Vec<&Expression> = principal_query(statement).into_iter().collect();
    pending.extend(statement_relations(statement));
    let mut placed: Vec<&Expression> = Vec::new();
    while let Some(query) = pending.pop() {
        placed.push(query);
        let mut parts: Vec<&Expression> = vec![query];
        while let Some(part) = parts.pop() {
            pending.extend(
                part.with_clause()
                    .iter()
                    .flat_map(|with| &with.ctes)
                    .map(|cte| &cte.this),
            );
            match unwrapped(part) {
                Expression::Subquery(subquery) => parts.push(&subquery.this),
                Expression::Select(select) => {
                    let items = select.from.iter().flat_map(|from| &from.expressions);
                    let items = items.chain(select.joins.iter().map(|join| &join.this));
                    pending.extend(items.flat_map(derived_queries));
                }
                body => {
                    if let Some(set) = body.set_operation() {
                        parts.extend([set.right, set.left]);
                    }
                }
            }
        }
    }
    placed
}

/// The CTE bodies a write owns, and the derived tables it reads from.
fn statement_relations(statement: &Expression) -> Vec<&Expression> {
    let (from, joins) = match statement {
        Expression::Update(update) => (update.from_clause.as_ref(), update.from_joins.as_slice()),
        Expression::Merge(merge) => return derived_queries(&merge.using),
        Expression::Delete(delete) => (None, delete.joins.as_slice()),
        Expression::Insert(_) => (None, &[][..]),
        _ => return Vec::new(),
    };
    let mut found: Vec<&Expression> = statement
        .with_clause()
        .iter()
        .flat_map(|with| &with.ctes)
        .map(|cte| &cte.this)
        .collect();
    let items = from.iter().flat_map(|from| &from.expressions);
    found.extend(
        items
            .chain(joins.iter().map(|join| &join.this))
            .flat_map(derived_queries),
    );
    found
}

/// The queries FROM item `item` derives its rows from.
fn derived_queries(item: &Expression) -> Vec<&Expression> {
    let mut pending: Vec<&Expression> = vec![item];
    let mut found: Vec<&Expression> = Vec::new();
    while let Some(node) = pending.pop() {
        match unwrapped(node) {
            Expression::Subquery(subquery) => found.push(&subquery.this),
            Expression::Alias(alias) => pending.push(&alias.this),
            Expression::Pivot(pivot) => pending.push(&pivot.this),
            Expression::Lateral(lateral) => pending.push(&lateral.this),
            Expression::JoinedTable(joined) => {
                pending.push(&joined.left);
                pending.extend(joined.joins.iter().map(|join| &join.this));
            }
            query @ (Expression::Select(_)
            | Expression::Union(_)
            | Expression::Intersect(_)
            | Expression::Except(_)) => found.push(query),
            _ => {}
        }
    }
    found
}
