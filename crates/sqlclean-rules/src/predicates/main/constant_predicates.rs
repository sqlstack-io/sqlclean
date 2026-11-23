//! `constant_predicates`: SQLCLEAN014's comparisons of one constant with itself.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::names::models::Literal;

use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::statement_nodes::statement_nodes;

/// The `=` and `<>` whose two whole operands are the same constant.
pub fn constant_predicates(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .filter_map(|node| match node {
            Expression::Eq(operation) | Expression::Neq(operation)
                if is_same_constant(&operation.left, &operation.right) =>
            {
                operation.operator_span.operator()
            }
            _ => None,
        })
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}

fn is_same_constant(left: &Expression, right: &Expression) -> bool {
    match (left, right) {
        (Expression::Literal(left), Expression::Literal(right)) => {
            left == right && matches!(left.as_ref(), Literal::Number(_) | Literal::String(_))
        }
        (Expression::Boolean(left), Expression::Boolean(right)) => left.value == right.value,
        _ => false,
    }
}
