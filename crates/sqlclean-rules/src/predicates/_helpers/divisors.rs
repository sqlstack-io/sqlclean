//! Divisions and their literal divisors.

use sqlquack_syntax::Expression;
use sqlquack_syntax::tokens::models::Span;

use crate::trees::main::number_text::number_text;
use crate::trees::main::unwrapped::unwrapped;

const DIVISION_CALL: &str = "DIV";
const MODULO_CALL: &str = "MOD";

/// The range of `node` when it divides by a literal zero.
pub(crate) fn zero_division(node: &Expression) -> Option<Span> {
    match node {
        Expression::Div(operation) | Expression::Mod(operation) => operation
            .operator_span
            .operation()
            .filter(|_| is_zero(&operation.right)),
        Expression::IntDiv(operation) => operation
            .operator_span
            .operation()
            .filter(|_| is_zero(&operation.expression)),
        Expression::Function(call) if call.qualified_name.len() <= 1 => {
            let [_, divisor] = call.args.as_slice() else {
                return None;
            };
            if !is_zero(divisor) {
                return None;
            }
            if call.name.eq_ignore_ascii_case(MODULO_CALL) {
                return call.span.0;
            }
            let operator: Span = call.name_span.0?;
            let operand: Span = call.argument_span.0?;
            (call.name.eq_ignore_ascii_case(DIVISION_CALL) && call.span.is_absent())
                .then(|| operator.through(operand))
        }
        _ => None,
    }
}

/// Whether `divisor` is a number literal, signed or not, whose digits are all zero.
fn is_zero(divisor: &Expression) -> bool {
    let number: &Expression = match unwrapped(divisor) {
        Expression::Neg(sign) | Expression::Pos(sign) => &sign.this,
        other => other,
    };
    number_text(number).is_some_and(all_zero)
}

fn all_zero(text: &str) -> bool {
    let mantissa: &str = text.split(['e', 'E']).next().unwrap_or(text);
    mantissa.chars().any(|digit| digit.is_ascii_digit())
        && mantissa
            .chars()
            .all(|digit| matches!(digit, '0' | '.' | '_'))
}
