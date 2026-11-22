//! `unknown_qualifiers`: SQLCLEAN029's qualifiers that name no relation in sight.

use crate::models::{RuleFinding, RuleInput};
use crate::relations::_helpers::references::references;
use crate::relations::_helpers::visibility::visible_names;

/// The qualifiers of column references that match no visible relation; a function's schema is none.
pub fn unknown_qualifiers(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        let scopes = statement.query_scopes();
        for (index, scope) in scopes.scopes.iter().enumerate() {
            if scope.select.from.is_none() {
                continue;
            }
            let qualified = references(input, scope).qualified;
            if qualified.is_empty() {
                continue;
            }
            let known: Vec<&str> = visible_names(&scopes, index);
            for column in qualified {
                if let Some(table) = &column.table
                    && let Some(span) = table.span
                    && !known
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(&table.name))
                {
                    found.push(RuleFinding { span, fix: None });
                }
            }
        }
    }
    found
}
