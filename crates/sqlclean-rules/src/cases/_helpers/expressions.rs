//! The CASE expressions of the statements, and the values their branches give.

use sqlquack_syntax::expressions::operators::models::Case;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::RuleInput;
use crate::trees::main::unwrapped::unwrapped;

/// Every CASE expression of the statements.
pub(crate) fn case_expressions<'a>(input: &RuleInput<'a>) -> Vec<&'a Case> {
    input
        .statements
        .iter()
        .flat_map(|statement| statement.dfs())
        .filter_map(|node| match node {
            Expression::Case(case) => Some(&**case),
            _ => None,
        })
        .collect()
}

/// Whether `value` is the boolean literal `expected` alone.
pub(crate) fn is_boolean(value: &Expression, expected: bool) -> bool {
    matches!(unwrapped(value), Expression::Boolean(boolean) if boolean.value == expected)
}
