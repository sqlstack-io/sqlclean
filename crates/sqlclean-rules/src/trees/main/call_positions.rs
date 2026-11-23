//! `call_positions`: where a call is written.

use sqlquack_syntax::Expression;
use sqlquack_syntax::tokens::models::SourceSpan;

/// A call's name and its marked argument as written; none for any other node.
pub(crate) fn call_positions(expression: &Expression) -> (SourceSpan, SourceSpan) {
    match expression {
        Expression::Function(call) => (call.name_span, call.argument_span),
        Expression::AggregateFunction(call) => (call.name_span, call.argument_span),
        _ => (SourceSpan::default(), SourceSpan::default()),
    }
}
