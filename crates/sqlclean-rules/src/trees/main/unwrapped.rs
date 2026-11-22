//! `unwrapped`: what an expression holds.

use sqlquack_syntax::Expression;

/// `expression` without the parentheses and comments around it.
pub(crate) fn unwrapped(mut expression: &Expression) -> &Expression {
    loop {
        expression = match expression {
            Expression::Paren(node) => &node.this,
            Expression::Annotated(node) => &node.this,
            _ => return expression,
        };
    }
}
