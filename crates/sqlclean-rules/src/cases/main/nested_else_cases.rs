//! `nested_else_cases`: SQLCLEAN031's CASEs that are an ELSE's whole value.

use sqlquack_syntax::Expression;

use crate::cases::_helpers::expressions::case_expressions;
use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::unwrapped::unwrapped;

/// The CASEs that are the whole value of another CASE's ELSE, at their keyword.
pub fn nested_else_cases(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    case_expressions(input)
        .into_iter()
        .filter_map(|case| match unwrapped(case.else_.as_ref()?) {
            Expression::Case(nested) => nested.case_keyword(),
            _ => None,
        })
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}
