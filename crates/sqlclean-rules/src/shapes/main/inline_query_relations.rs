//! `inline_query_relations`: SQLCLEAN039's queries written as relations.

use sqlquack_syntax::Expression;

use crate::models::{RuleFinding, RuleInput};
use crate::shapes::_helpers::derived_tables::derived_tables;
use crate::trees::main::placed_queries::placed_queries;
use crate::trees::main::unwrapped::unwrapped;

/// The derived tables of every step, leaving out queries inside expressions and LATERAL ones.
pub fn inline_query_relations(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        for query in placed_queries(statement) {
            let mut parts: Vec<&Expression> = vec![query];
            while let Some(part) = parts.pop() {
                match unwrapped(part) {
                    Expression::Subquery(subquery) => parts.push(&subquery.this),
                    Expression::Select(select) => found.extend(
                        derived_tables(select)
                            .into_iter()
                            .map(|span| RuleFinding { span, fix: None }),
                    ),
                    body => parts.extend(
                        body.set_operation()
                            .into_iter()
                            .flat_map(|set| [set.right, set.left]),
                    ),
                }
            }
        }
    }
    found
}
