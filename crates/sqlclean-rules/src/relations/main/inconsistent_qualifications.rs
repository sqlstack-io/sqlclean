//! `inconsistent_qualifications`: SQLCLEAN028's scopes that qualify only some columns.

use crate::models::{RuleFinding, RuleInput};
use crate::relations::_helpers::references::references;
use crate::relations::_helpers::styles::inconsistent;

/// The unqualified references of one-relation scopes that also qualify, each judged as it was written.
pub fn inconsistent_qualifications(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        for scope in statement.query_scopes().scopes {
            let Some(from) = &scope.select.from else {
                continue;
            };
            let items: usize = from.expressions.len() + scope.select.joins.len();
            if items.max(scope.relations.len()) > 1 {
                continue;
            }
            let written = references(input, &scope);
            found.extend(
                inconsistent(input, (&written.qualified, &written.unqualified))
                    .into_iter()
                    .map(|span| RuleFinding { span, fix: None }),
            );
        }
    }
    found
}
