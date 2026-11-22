//! `query_within`: the query inside parentheses.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::Subquery;

/// The query within `expression`'s parentheses and unaliased subquery nodes.
pub(crate) fn query_within(expression: &Expression) -> &Expression {
    match expression {
        Expression::Paren(paren) => query_within(&paren.this),
        Expression::Subquery(subquery) if around(subquery) => query_within(&subquery.this),
        other => other,
    }
}

/// Whether `subquery` only wraps a query: unaliased, without a sample or watermark.
fn around(subquery: &Subquery) -> bool {
    subquery.alias.is_none() && subquery.table_sample.is_none() && subquery.watermark.is_none()
}
