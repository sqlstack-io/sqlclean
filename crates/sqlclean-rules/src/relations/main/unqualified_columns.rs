//! `unqualified_columns`: SQLCLEAN027's columns that leave their relation unsaid.

use crate::models::{RuleFinding, RuleInput};
use crate::relations::_helpers::references::references;
use crate::relations::_helpers::unqualified::{is_macro_parameter, is_merged, using_columns};

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
            let sources: usize =
                items.max(scope.relations.len()) + scope.select.lateral_views.len();
            if sources <= 1 {
                continue;
            }
            let merged: Vec<&str> = using_columns(scope.select);
            found.extend(
                references(input, &scope)
                    .unqualified
                    .into_iter()
                    .filter(|column| {
                        !is_merged(&merged, column) && !is_macro_parameter(input, column)
                    })
                    .filter_map(|column| column.name.span.or(column.span))
                    .map(|span| RuleFinding { span, fix: None }),
            );
        }
    }
    found
}
