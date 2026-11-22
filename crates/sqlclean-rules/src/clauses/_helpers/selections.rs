//! Which rows LIMIT and OFFSET select.

use sqlquack_syntax::tokens::models::Span;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::clauses::_helpers::orders::query_order;
use crate::trees::main::number_text::number_text;
use crate::trees::main::unwrapped::unwrapped;

/// The keyword of `node`'s LIMIT or OFFSET when the rows it selects depend on an order nobody wrote.
pub(crate) fn unordered_selection(node: &Expression) -> Option<Span> {
    let (limit, offset, keyword) = match node {
        Expression::Select(query) => (
            query.limit.as_ref().map(|limit| &limit.this),
            query.offset.is_some(),
            query.row_limit_span,
        ),
        Expression::Subquery(query) => (
            query.limit.as_ref().map(|limit| &limit.this),
            query.offset.is_some(),
            query.row_limit_span,
        ),
        Expression::Union(query) => (
            query.limit.as_deref(),
            query.offset.is_some(),
            query.row_limit_span,
        ),
        Expression::Intersect(query) => (
            query.limit.as_deref(),
            query.offset.is_some(),
            query.row_limit_span,
        ),
        Expression::Except(query) => (
            query.limit.as_deref(),
            query.offset.is_some(),
            query.row_limit_span,
        ),
        _ => return None,
    };
    if limit.is_none() && !offset || limit.is_some_and(is_none_selected) {
        return None;
    }
    let mut query: &Expression = node;
    loop {
        if query_order(query).is_some() {
            return None;
        }
        query = match query {
            Expression::Subquery(inner) => unwrapped(&inner.this),
            _ => break,
        };
    }
    if selects_only_constants(query) {
        return None;
    }
    keyword.0
}

fn is_none_selected(count: &Expression) -> bool {
    number_text(count).is_some_and(|text| text.parse::<u64>() == Ok(0))
}

/// Whether every row of `query` is the same constants.
fn selects_only_constants(query: &Expression) -> bool {
    match query {
        Expression::Select(select) => {
            !select.expressions.is_empty() && select.expressions.iter().all(is_constant)
        }
        _ => false,
    }
}

fn is_constant(item: &Expression) -> bool {
    let value: &Expression = match item {
        Expression::Alias(alias) => &alias.this,
        other => other,
    };
    value.dfs().all(|node| {
        matches!(
            node,
            Expression::Literal(_)
                | Expression::Boolean(_)
                | Expression::Null(_)
                | Expression::Paren(_)
                | Expression::Annotated(_)
                | Expression::Neg(_)
                | Expression::Pos(_)
                | Expression::Not(_)
                | Expression::Cast(_)
                | Expression::TryCast(_)
                | Expression::DataType(_)
                | Expression::Interval(_)
                | Expression::Tuple(_)
                | Expression::Array(_)
                | Expression::Case(_)
                | Expression::Add(_)
                | Expression::Sub(_)
                | Expression::Mul(_)
                | Expression::Div(_)
                | Expression::Mod(_)
                | Expression::Concat(_)
                | Expression::And(_)
                | Expression::Or(_)
                | Expression::Eq(_)
                | Expression::Neq(_)
                | Expression::Lt(_)
                | Expression::Lte(_)
                | Expression::Gt(_)
                | Expression::Gte(_)
        )
    })
}
