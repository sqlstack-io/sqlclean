//! `unordered_limits`: SQLCLEAN004's row selections without an order.

use crate::clauses::_helpers::selections::unordered_selection;
use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::statement_nodes::statement_nodes;

/// The LIMIT and OFFSET keywords of queries without ORDER BY whose rows could differ.
pub fn unordered_limits(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .filter_map(unordered_selection)
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}
