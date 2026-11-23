//! `redundant_else_nulls`: SQLCLEAN012's ELSE clauses that are NULL alone.

use sqlquack_syntax::Expression;

use crate::cases::_helpers::expressions::case_expressions;
use crate::models::{Replacement, RuleEdit, RuleFinding, RuleInput};
use crate::trees::main::unwrapped::unwrapped;

/// The ELSE clauses whose whole value is NULL, with their removal as the fix.
pub fn redundant_else_nulls(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    case_expressions(input)
        .into_iter()
        .filter(|case| {
            case.else_
                .as_ref()
                .is_some_and(|value| matches!(unwrapped(value), Expression::Null(_)))
        })
        .filter_map(|case| case.else_span.0)
        .map(|span| RuleFinding {
            span,
            fix: Some(RuleEdit {
                span,
                replacement: Replacement::Text(""),
                trailing_layout: false,
            }),
        })
        .collect()
}
