//! BETWEEN bounds whose order is exact.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::names::models::Literal;
use sqlquack_syntax::expressions::types::models::DataType;

use crate::trees::main::number_text::number_text;
use crate::trees::main::unwrapped::unwrapped;

const ISO_DATE_LENGTH: usize = 10;
const DATE_SEPARATORS: [usize; 2] = [4, 7];

/// Whether `low` is above `high`, both being numbers or both ISO dates.
pub(crate) fn is_reversed(low: &Expression, high: &Expression) -> bool {
    match (number(low), number(high), date(low), date(high)) {
        (Some(low), Some(high), _, _) => low > high,
        (_, _, Some(low), Some(high)) => low > high,
        _ => false,
    }
}

fn number(bound: &Expression) -> Option<f64> {
    let (sign, digits): (f64, &Expression) = match unwrapped(bound) {
        Expression::Neg(negation) => (-1.0, &negation.this),
        Expression::Pos(positive) => (1.0, &positive.this),
        other => (1.0, other),
    };
    let text: String = number_text(digits)?.replace('_', "");
    match text.parse::<f64>() {
        Ok(magnitude) => Some(sign * magnitude),
        Err(_) => None,
    }
}

fn date(bound: &Expression) -> Option<&str> {
    let literal: &Literal = match bound {
        Expression::Literal(literal) if matches!(literal.as_ref(), Literal::Date(_)) => literal,
        Expression::Cast(cast)
            if !cast.double_colon_syntax && matches!(cast.to, DataType::Date) =>
        {
            match &cast.this {
                Expression::Literal(literal) => literal,
                _ => return None,
            }
        }
        _ => return None,
    };
    match literal {
        Literal::Date(text) | Literal::String(text) if is_iso(text) => Some(text),
        _ => None,
    }
}

fn is_iso(text: &str) -> bool {
    text.len() == ISO_DATE_LENGTH
        && text.char_indices().all(
            |(index, character)| match DATE_SEPARATORS.contains(&index) {
                true => character == '-',
                false => character.is_ascii_digit(),
            },
        )
}
