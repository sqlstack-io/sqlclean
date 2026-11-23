//! The clauses that run before aggregates and windows are computed.

use sqlquack_syntax::tokens::models::Span;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{DialectFacts, RuleFinding};
use crate::trees::main::call_name::call_name;
use crate::trees::main::is_aggregate::is_aggregate;
use crate::trees::main::is_foreign_aggregate::is_foreign_aggregate;
use crate::trees::main::node_joins::node_joins;

const SINGLE_ARGUMENT_AGGREGATES: [&str; 2] = ["MIN", "MAX"];

/// WHERE, ON, GROUP BY and an aggregate's FILTER as `node` writes them.
pub(crate) fn early_clauses(node: &Expression) -> Vec<&Expression> {
    let mut clauses: Vec<&Expression> = node_joins(node)
        .into_iter()
        .filter_map(|join| join.on.as_ref())
        .collect();
    let condition = match node {
        Expression::Select(select) => {
            clauses.extend(select.group_by.iter().flat_map(|group| &group.expressions));
            select.where_clause.as_ref()
        }
        Expression::Update(update) => update.where_clause.as_ref(),
        Expression::Delete(delete) => delete.where_clause.as_ref(),
        Expression::AggregateFunction(call) => {
            clauses.extend(call.filter.as_ref());
            None
        }
        Expression::Merge(merge) => {
            clauses.extend(merge.on.as_deref());
            None
        }
        _ => None,
    };
    clauses.extend(condition.map(|condition| &condition.this));
    clauses
}

/// The aggregate and window calls directly in `clause`, outside the queries inside it.
pub(crate) fn computed_later(clause: &Expression, dialect: &DialectFacts) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    let mut pending: Vec<&Expression> = vec![clause];
    while let Some(node) = pending.pop() {
        match node {
            Expression::Select(_)
            | Expression::Subquery(_)
            | Expression::Union(_)
            | Expression::Intersect(_)
            | Expression::Except(_)
            | Expression::Values(_) => {}
            Expression::WindowFunction(window) => {
                found.extend(window.span.0.map(|span| RuleFinding { span, fix: None }));
            }
            _ if is_known_aggregate(node, dialect) => {
                found.extend(written_call(node).map(|span| RuleFinding { span, fix: None }));
            }
            _ => pending.extend(node.children()),
        }
    }
    found
}

/// A call's range, which a keyword call keeps apart from its node's.
fn written_call(node: &Expression) -> Option<Span> {
    match node {
        Expression::KeywordCall(call) => call.call_span.0,
        _ => node.source_span(),
    }
}

/// An aggregate here, leaving out other dialects' names and MIN or MAX of several values.
fn is_known_aggregate(node: &Expression, dialect: &DialectFacts) -> bool {
    if !is_aggregate(node, dialect) {
        return false;
    }
    let Some(name) = call_name(node) else {
        return true;
    };
    let arguments: usize = match node {
        Expression::Function(call) => call.args.len(),
        Expression::AggregateFunction(call) => call.args.len(),
        _ => 0,
    };
    let scalar: bool = arguments > 1
        && SINGLE_ARGUMENT_AGGREGATES
            .iter()
            .any(|known| known.eq_ignore_ascii_case(name));
    !scalar && (dialect.generic || !is_foreign_aggregate(name, dialect))
}
