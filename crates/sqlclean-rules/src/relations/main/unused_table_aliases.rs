//! `unused_table_aliases`: SQLCLEAN023's aliases nothing reads.

use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{RuleFinding, RuleInput};
use crate::relations::_helpers::aliases::unused_aliases;

/// The relation aliases never used in their query, with their removal as the fix where it is safe.
pub fn unused_table_aliases(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        let scopes = statement.query_scopes();
        for node in statement.dfs() {
            let Expression::Select(select) = node else {
                continue;
            };
            for scope in &scopes.scopes {
                if std::ptr::eq(scope.select, &**select) {
                    found.extend(unused_aliases(scope, node));
                }
            }
        }
    }
    found
}
