//! `recursive_ctes`: SQLCLEAN037's recursive WITHs.

use sqlquack_syntax::Expression;

use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::statement_nodes::statement_nodes;

/// The WITH keyword of every WITH RECURSIVE.
pub fn recursive_ctes(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .filter_map(Expression::with_clause)
        .filter(|with| with.recursive)
        .filter_map(|with| with.keyword_span.0)
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}
