//! Comparisons with NULL.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::operators::models::BinaryOp;

use crate::models::{Replacement, RuleEdit, RuleFinding};
use crate::trees::main::unwrapped::unwrapped;

/// The comparison `node` is.
pub(crate) fn comparison(node: &Expression) -> Option<&BinaryOp> {
    match node {
        Expression::Eq(operation)
        | Expression::Neq(operation)
        | Expression::Lt(operation)
        | Expression::Lte(operation)
        | Expression::Gt(operation)
        | Expression::Gte(operation) => Some(operation),
        _ => None,
    }
}

/// The finding when NULL is written beside comparison `node`'s operator.
pub(crate) fn null_comparison(node: &Expression) -> Option<RuleFinding> {
    let operation: &BinaryOp = comparison(node)?;
    if !(starts_with_null(&operation.right) || ends_with_null(&operation.left)) {
        return None;
    }
    let span = operation.operator_span.operator()?;
    Some(RuleFinding {
        span,
        fix: null_test(node, operation),
    })
}

/// IS or IS NOT in place of `=` or `<>`, only when its whole right side is NULL.
fn null_test(node: &Expression, operation: &BinaryOp) -> Option<RuleEdit> {
    let (word, test): (&str, &str) = match node {
        Expression::Eq(_) => ("IS", "IS NULL"),
        Expression::Neq(_) => ("IS NOT", "IS NOT NULL"),
        _ => return None,
    };
    if !matches!(unwrapped(&operation.right), Expression::Null(_)) {
        return None;
    }
    let (span, replacement) = match &operation.right {
        Expression::Paren(_) => (operation.operator_span.operation()?, test),
        _ => (operation.operator_span.operator()?, word),
    };
    Some(RuleEdit {
        span,
        replacement: Replacement::Spaced(replacement),
        trailing_layout: false,
    })
}

fn starts_with_null(operand: &Expression) -> bool {
    let mut node: &Expression = operand;
    loop {
        if matches!(unwrapped(node), Expression::Null(_)) {
            return true;
        }
        node = match node {
            Expression::Add(operation)
            | Expression::Sub(operation)
            | Expression::Mul(operation)
            | Expression::Div(operation)
            | Expression::Mod(operation)
            | Expression::Concat(operation) => &operation.left,
            Expression::Cast(cast) if cast.double_colon_syntax => &cast.this,
            Expression::Annotated(annotated) => &annotated.this,
            _ => return false,
        };
    }
}

fn ends_with_null(operand: &Expression) -> bool {
    let mut node: &Expression = operand;
    loop {
        if matches!(unwrapped(node), Expression::Null(_)) {
            return true;
        }
        node = match node {
            Expression::Add(operation)
            | Expression::Sub(operation)
            | Expression::Mul(operation)
            | Expression::Div(operation)
            | Expression::Mod(operation)
            | Expression::Concat(operation) => &operation.right,
            Expression::Neg(operation) | Expression::Pos(operation) => &operation.this,
            Expression::Annotated(annotated) => &annotated.this,
            _ => return false,
        };
    }
}
