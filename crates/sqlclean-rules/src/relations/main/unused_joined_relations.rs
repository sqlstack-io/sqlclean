//! `unused_joined_relations`: SQLCLEAN032's joins that contribute no values.

use crate::models::{RuleFinding, RuleInput};
use crate::relations::_helpers::references::references;
use crate::relations::_helpers::unread::{projects_bare_star, unread_joins};

/// The JOINs whose relation nothing after them reads, where every reference says what it reads.
pub fn unused_joined_relations(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        for scope in statement.query_scopes().scopes {
            if projects_bare_star(&scope) {
                continue;
            }
            if references(input, &scope).unqualified.is_empty() {
                found.extend(unread_joins(&scope));
            }
        }
    }
    found
}
