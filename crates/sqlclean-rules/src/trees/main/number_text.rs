//! `number_text`: a number literal's text.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::names::models::Literal;

/// The text of `expression` when it is a number literal alone.
pub(crate) fn number_text(expression: &Expression) -> Option<&str> {
    match expression {
        Expression::Literal(literal) => match literal.as_ref() {
            Literal::Number(text) => Some(text),
            _ => None,
        },
        _ => None,
    }
}
