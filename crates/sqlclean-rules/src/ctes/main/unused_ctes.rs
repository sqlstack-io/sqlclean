//! `unused_ctes`: SQLCLEAN005's CTEs the query never reaches.

use crate::ctes::_helpers::reachability::{query_with, unreached};
use crate::ctes::_helpers::removals::removal;
use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::statement_nodes::statement_nodes;

/// The names of CTEs no read reaches, with their removal as the fix where it is safe.
pub fn unused_ctes(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for node in statement_nodes(input) {
        let Some(with) = query_with(node) else {
            continue;
        };
        for index in unreached(input, (node, with)) {
            found.extend(with.ctes[index].alias.span.map(|span| RuleFinding {
                span,
                fix: removal(with, index),
            }));
        }
    }
    found
}
