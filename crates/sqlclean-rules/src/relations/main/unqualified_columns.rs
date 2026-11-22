//! `unqualified_columns`: SQLCLEAN027's columns that leave their relation unsaid.

use crate::models::{RuleFinding, RuleInput};
use crate::relations::_helpers::references::references;

/// The unqualified column references of scopes that read several relations.
pub fn unqualified_columns(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        for scope in statement.query_scopes().scopes {
            let items: usize = scope.select.joins.len()
                + scope
                    .select
                    .from
                    .as_ref()
                    .map_or(0, |from| from.expressions.len());
            if items.max(scope.relations.len()) <= 1 {
                continue;
            }
            found.extend(
                references(input, &scope)
                    .unqualified
                    .into_iter()
                    .filter_map(|column| column.name.span.or(column.span))
                    .map(|span| RuleFinding { span, fix: None }),
            );
        }
    }
    found
}
