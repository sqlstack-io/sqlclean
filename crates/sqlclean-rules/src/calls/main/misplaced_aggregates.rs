//! `misplaced_aggregates`: SQLCLEAN053's aggregates and windows in clauses that run before them.

use crate::calls::_helpers::clauses::{computed_later, early_clauses};
use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::statement_nodes::statement_nodes;

/// The aggregate and window calls directly in WHERE, ON or GROUP BY.
pub fn misplaced_aggregates(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .flat_map(early_clauses)
        .flat_map(|clause| computed_later(clause, &input.dialect))
        .collect()
}
