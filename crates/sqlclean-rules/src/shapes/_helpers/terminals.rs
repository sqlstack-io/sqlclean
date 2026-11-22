//! The step a body ends on.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::names::models::TableRef;
use sqlquack_syntax::expressions::queries::models::{Select, With};
use sqlquack_syntax::tokens::models::Span;

use crate::models::RuleInput;
use crate::trees::main::principal_query::principal_query;
use crate::trees::main::unwrapped::unwrapped;

/// A body's terminal step and the WITH before it.
#[derive(Clone, Copy)]
pub(crate) struct Terminal<'a> {
    pub(crate) with: Option<&'a With>,
    /// The first operator of a terminal set operation.
    pub(crate) operator: Option<Span>,
    pub(crate) select: Option<&'a Select>,
}

/// The terminal step of the SQL's first statement that has a query.
pub(crate) fn terminal<'a>(input: &RuleInput<'a>) -> Option<Terminal<'a>> {
    let (statement, query): (&Expression, &Expression) = input
        .statements
        .iter()
        .find_map(|statement| Some((statement, principal_query(statement)?)))?;
    let with: Option<&With> = statement.with_clause().or_else(|| query.with_clause());
    let mut operator: Option<Span> = None;
    let mut step: &Expression = query;
    while let Some(set) = step.set_operation() {
        operator = set.operator_span.0;
        step = unwrapped(set.left);
    }
    Some(Terminal {
        with,
        operator,
        select: match step {
            Expression::Select(select) if operator.is_none() => Some(select),
            _ => None,
        },
    })
}

/// Whether the terminal SELECT does work of its own.
pub(crate) fn has_logic(select: &Select) -> bool {
    select
        .joins
        .iter()
        .any(|join| join.kind != sqlquack_syntax::expressions::queries::models::JoinKind::Implicit)
        || select
            .from
            .iter()
            .flat_map(|from| &from.expressions)
            .any(|item| matches!(unwrapped(item), Expression::JoinedTable(_)))
        || select.where_clause.is_some()
        || select.group_by.is_some()
        || select.having.is_some()
        || select.qualify.is_some()
        || select.order_by.is_some()
        || select.limit.is_some()
        || select.offset.is_some()
}

/// Whether `select` only lists columns of the last CTE of `with`.
pub(crate) fn reads_last_cte(select: &Select, with: &With) -> bool {
    let (Some(last), Some(from)) = (with.ctes.last(), &select.from) else {
        return false;
    };
    let [Expression::Table(table)] = from.expressions.as_slice() else {
        return false;
    };
    !select.expressions.is_empty()
        && select.expressions.iter().all(is_plain_output)
        && select.joins.is_empty()
        && !has_logic(select)
        && select.fetch.is_none()
        && select.windows.is_none()
        && select.sample.is_none()
        && select.lateral_views.is_empty()
        && select.sort_by.is_none()
        && select.distribute_by.is_none()
        && select.cluster_by.is_none()
        && is_bare_table(table)
        && table.name.name.eq_ignore_ascii_case(&last.alias.name)
}

fn is_bare_table(table: &TableRef) -> bool {
    table.schema.is_none()
        && table.catalog.is_none()
        && table.alias.is_none()
        && table.when.is_none()
        && table.changes.is_none()
        && table.version.is_none()
        && table.table_sample.is_none()
        && table.sample_call.is_none()
}

fn is_plain_output(item: &Expression) -> bool {
    let value: &Expression = match item {
        Expression::Alias(alias) => &alias.this,
        other => other,
    };
    matches!(
        unwrapped(value),
        Expression::Column(_) | Expression::Identifier(_) | Expression::Star(_)
    ) && !matches!(value, Expression::Paren(_))
}
