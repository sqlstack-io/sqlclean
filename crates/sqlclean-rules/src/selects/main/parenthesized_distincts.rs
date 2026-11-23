//! `parenthesized_distincts`: SQLCLEAN013's DISTINCT written like a function.

use sqlquack_syntax::Expression;

use crate::models::{RuleFinding, RuleInput};
use crate::selects::_helpers::distincts::{call_distinct, select_distinct};
use crate::trees::main::statement_nodes::statement_nodes;

/// The DISTINCT of a select list or call written before parentheses.
pub fn parenthesized_distincts(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .filter_map(|node| match node {
            Expression::Select(select) => select_distinct(select),
            _ => call_distinct(node),
        })
        .collect()
}
