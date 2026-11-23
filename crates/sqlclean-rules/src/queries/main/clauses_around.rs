//! `clauses_around`: the clauses around a query.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::Subquery;

/// The clause expressions of the unaliased subquery nodes around `expression`'s query.
pub(crate) fn clauses_around(expression: &Expression) -> Vec<&Expression> {
    let mut clauses: Vec<&Expression> = Vec::new();
    let mut node: &Expression = expression;
    loop {
        node = match node {
            Expression::Paren(paren) => &paren.this,
            Expression::Subquery(subquery) if around(subquery) => {
                let ordered = subquery
                    .order_by
                    .iter()
                    .flat_map(|order| &order.expressions)
                    .chain(subquery.sort_by.iter().flat_map(|sort| &sort.expressions))
                    .chain(
                        subquery
                            .cluster_by
                            .iter()
                            .flat_map(|cluster| &cluster.expressions),
                    );
                clauses.extend(ordered.map(|ordered| &ordered.this));
                clauses.extend(
                    subquery
                        .distribute_by
                        .iter()
                        .flat_map(|distribute| &distribute.expressions),
                );
                clauses.extend(subquery.limit.iter().map(|limit| &limit.this));
                clauses.extend(subquery.offset.iter().map(|offset| &offset.this));
                &subquery.this
            }
            _ => return clauses,
        };
    }
}

/// Whether `subquery` only wraps a query: unaliased, without a sample or watermark.
fn around(subquery: &Subquery) -> bool {
    subquery.alias.is_none() && subquery.table_sample.is_none() && subquery.watermark.is_none()
}
