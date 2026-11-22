//! `unfiltered_writes`: SQLCLEAN049's writes that change every row.

use sqlquack_syntax::expressions::queries::models::Join;
use sqlquack_syntax::expressions::statements::models::{Delete, Update};
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{RuleFinding, RuleInput};

/// The UPDATE and DELETE statements with nothing that limits the rows they change.
pub fn unfiltered_writes(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    input
        .statements
        .iter()
        .filter_map(|statement| match statement {
            Expression::Update(update) if !update_is_limited(update) => update.keyword_span.0,
            Expression::Delete(delete) if !delete_is_limited(delete) => delete.keyword_span.0,
            _ => None,
        })
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}

/// Whether a WHERE, LIMIT or join on the target limits an UPDATE; a join inside FROM cannot read it.
fn update_is_limited(update: &Update) -> bool {
    update
        .where_clause
        .as_ref()
        .is_some_and(|clause| reads_rows(&clause.this))
        || update.limit.is_some()
        || update.table_joins.iter().any(join_is_limiting)
}

fn delete_is_limited(delete: &Delete) -> bool {
    delete
        .where_clause
        .as_ref()
        .is_some_and(|clause| reads_rows(&clause.this))
        || delete.limit.is_some()
        || delete.joins.iter().any(join_is_limiting)
}

/// Whether a predicate depends on the rows: it reads a column or a subquery, unlike `WHERE TRUE`.
fn reads_rows(predicate: &Expression) -> bool {
    predicate.dfs().any(|node| {
        matches!(
            node,
            Expression::Column(_) | Expression::Select(_) | Expression::Subquery(_)
        )
    })
}

fn join_is_limiting(join: &Join) -> bool {
    join.on.as_ref().is_some_and(reads_rows) || !join.using.is_empty()
}
