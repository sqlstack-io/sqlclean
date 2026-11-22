//! `null_comparisons`: SQLCLEAN001's comparisons with NULL.

use crate::models::{RuleFinding, RuleInput};
use crate::predicates::_helpers::comparisons::null_comparison;
use crate::trees::main::statement_nodes::statement_nodes;

/// The comparisons with NULL beside their operator, with IS or IS NOT as the fix where that is exact.
pub fn null_comparisons(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input).filter_map(null_comparison).collect()
}
