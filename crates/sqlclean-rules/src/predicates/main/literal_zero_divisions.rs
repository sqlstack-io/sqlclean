//! `literal_zero_divisions`: SQLCLEAN048's divisions by a literal zero.

use crate::models::{RuleFinding, RuleInput};
use crate::predicates::_helpers::divisors::zero_division;
use crate::trees::main::statement_nodes::statement_nodes;

/// The divisions and MOD calls whose whole divisor is a literal zero.
pub fn literal_zero_divisions(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .filter_map(zero_division)
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}
