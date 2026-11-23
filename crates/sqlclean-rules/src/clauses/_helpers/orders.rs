//! Where a query keeps its ORDER BY.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::OrderBy;

/// The ORDER BY `node` owns as a query.
pub(crate) fn query_order(node: &Expression) -> Option<&OrderBy> {
    match node {
        Expression::Select(query) => query.order_by.as_ref(),
        Expression::Union(query) => query.order_by.as_ref(),
        Expression::Intersect(query) => query.order_by.as_ref(),
        Expression::Except(query) => query.order_by.as_ref(),
        Expression::Subquery(query) => query.order_by.as_ref(),
        _ => None,
    }
}

/// Whether `order` states a direction for some items and none for others.
pub(crate) fn states_some_directions(order: &OrderBy) -> bool {
    let stated: usize = order
        .expressions
        .iter()
        .filter(|item| item.desc || item.explicit_asc)
        .count();
    stated > 0 && stated < order.expressions.len()
}
