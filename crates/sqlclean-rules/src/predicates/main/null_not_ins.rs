//! `null_not_ins`: SQLCLEAN019's NOT IN lists that hold NULL.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::operators::models::In;
use sqlquack_syntax::tokens::models::Span;

use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::statement_nodes::statement_nodes;
use crate::trees::main::unwrapped::unwrapped;

/// The negated IN lists that hold NULL, on NOT where it is written before IN and otherwise on IN.
pub fn null_not_ins(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .filter_map(negated_list)
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}

fn negated_list(node: &Expression) -> Option<Span> {
    let list: &In = match node {
        Expression::In(list) if list.not => list,
        Expression::Not(negation) => match unwrapped(&negation.this) {
            Expression::In(list) if !list.not => list,
            _ => return None,
        },
        _ => return None,
    };
    let holds_null: bool =
        list.query.is_none() && list.expressions.iter().any(|value| is_null(value));
    list.operator_span.operator().filter(|_| holds_null)
}

fn is_null(value: &Expression) -> bool {
    match unwrapped(value) {
        Expression::Null(_) => true,
        Expression::Cast(cast) => matches!(unwrapped(&cast.this), Expression::Null(_)),
        _ => false,
    }
}
