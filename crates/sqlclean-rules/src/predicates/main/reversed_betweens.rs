//! `reversed_betweens`: SQLCLEAN052's BETWEEN bounds in the wrong order.

use sqlquack_syntax::Expression;

use crate::models::{RuleFinding, RuleInput};
use crate::predicates::_helpers::bounds::is_reversed;
use crate::trees::main::statement_nodes::statement_nodes;

/// The BETWEENs, other than SYMMETRIC ones, whose literal lower bound is above the upper.
pub fn reversed_betweens(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .filter_map(|node| match node {
            Expression::Between(test)
                if test.symmetric != Some(true) && is_reversed(&test.low, &test.high) =>
            {
                test.operator_span.operation()
            }
            _ => None,
        })
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}
