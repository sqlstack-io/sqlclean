//! Queries written in parentheses as relations.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::{JoinKind, Select};
use sqlquack_syntax::tokens::models::Span;

use crate::trees::main::unwrapped::unwrapped;

/// The first keyword of each derived table `select` reads that cannot see the items before it.
pub(crate) fn derived_tables(select: &Select) -> Vec<Span> {
    let mut items: Vec<&Expression> = select
        .from
        .iter()
        .flat_map(|from| &from.expressions)
        .collect();
    items.extend(
        select
            .joins
            .iter()
            .filter(|join| {
                !matches!(
                    join.kind,
                    JoinKind::CrossApply
                        | JoinKind::OuterApply
                        | JoinKind::Lateral
                        | JoinKind::LeftLateral
                )
            })
            .map(|join| &join.this),
    );
    let mut found: Vec<Span> = Vec::new();
    while let Some(item) = items.pop() {
        match item {
            Expression::Annotated(node) => items.push(&node.this),
            Expression::Alias(node) => items.push(&node.this),
            Expression::Pivot(node) => items.push(&node.this),
            Expression::JoinedTable(node) => {
                items.push(&node.left);
                items.extend(node.joins.iter().map(|join| &join.this));
            }
            Expression::Subquery(node) if !node.lateral => found.extend(first_keyword(&node.this)),
            Expression::Paren(node) => match unwrapped(&node.this) {
                joined @ Expression::JoinedTable(_) => items.push(joined),
                query => found.extend(first_keyword(query)),
            },
            _ => {}
        }
    }
    found
}

/// The keyword a query begins with, inside its parentheses.
fn first_keyword(query: &Expression) -> Option<Span> {
    let mut node: &Expression = query;
    loop {
        if let Some(with) = node.with_clause() {
            return with.keyword_span.0;
        }
        node = match node {
            Expression::Annotated(inner) => &inner.this,
            Expression::Paren(inner) => &inner.this,
            Expression::Subquery(inner) if inner.alias.is_none() => &inner.this,
            Expression::Select(select) => return select.select_span.0,
            Expression::Values(values) => return values.keyword_span.0,
            _ => node.set_operation()?.left,
        };
    }
}
