//! A query's projections, sources and columns.

use sqlquack_syntax::expressions::names::models::Column;
use sqlquack_syntax::expressions::names::models::Star;
use sqlquack_syntax::expressions::queries::models::{Select, Subquery};
use sqlquack_syntax::{Expression, ExpressionWalk};

/// The query within `expression`'s parentheses and unaliased subquery nodes.
pub(crate) fn query_within(expression: &Expression) -> &Expression {
    match expression {
        Expression::Paren(paren) => query_within(&paren.this),
        Expression::Subquery(subquery) if around(subquery) => query_within(&subquery.this),
        other => other,
    }
}

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

/// One projection of a select list.
pub(crate) enum Projection<'a> {
    Named {
        name: String,
        expression: &'a Expression,
    },
    Star(&'a Star),
    Unnamed,
}

pub(crate) fn lower(name: &str) -> String {
    name.to_ascii_lowercase()
}

pub(crate) fn projections(select: &Select) -> Vec<Projection<'_>> {
    select
        .expressions
        .iter()
        .map(|expression| match expression {
            Expression::Alias(alias) => Projection::Named {
                name: lower(&alias.alias.name),
                expression: &alias.this,
            },
            Expression::Column(column) => Projection::Named {
                name: lower(&column.name.name),
                expression,
            },
            Expression::Star(star) => Projection::Star(star),
            _ => Projection::Unnamed,
        })
        .collect()
}

pub(crate) fn columns(expression: &Expression) -> Vec<&Column> {
    expression
        .dfs()
        .filter_map(|node| match node {
            Expression::Column(column) => Some(column.as_ref()),
            _ => None,
        })
        .collect()
}

pub(crate) fn qualifier(column: &Column) -> Option<String> {
    column.table.as_ref().map(|table| lower(&table.name))
}

/// Relations read directly by one select: `(alias, relation name, expression)`.
pub(crate) fn sources(select: &Select) -> Vec<(String, Option<String>, &Expression)> {
    let mut found: Vec<(String, Option<String>, &Expression)> = Vec::new();
    let from = select.from.iter().flat_map(|from| from.expressions.iter());
    for expression in from
        .chain(select.joins.iter().map(|join| &join.this))
        .flat_map(joined_relations)
    {
        match expression {
            Expression::Table(table) => {
                let name: String = lower(&table.name.name);
                let alias: String = table
                    .alias
                    .as_ref()
                    .map_or_else(|| name.clone(), |alias| lower(&alias.name));
                found.push((alias, Some(name), expression));
            }
            Expression::Subquery(subquery) => {
                let alias: String = subquery
                    .alias
                    .as_ref()
                    .map_or_else(String::new, |alias| lower(&alias.name));
                found.push((alias, None, expression));
            }
            _ => found.push((String::new(), None, expression)),
        }
    }
    found
}

/// The relations of an unaliased joined table, in parentheses or not, in order; else `item`.
fn joined_relations(item: &Expression) -> Vec<&Expression> {
    match item {
        Expression::JoinedTable(joined) if joined.alias.is_none() => {
            let mut relations: Vec<&Expression> = joined_relations(&joined.left);
            for join in &joined.joins {
                relations.extend(joined_relations(&join.this));
            }
            relations
        }
        Expression::Paren(paren) if matches!(paren.this, Expression::JoinedTable(_)) => {
            joined_relations(&paren.this)
        }
        _ => vec![item],
    }
}
