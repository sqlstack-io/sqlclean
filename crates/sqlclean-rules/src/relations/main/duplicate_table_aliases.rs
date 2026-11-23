//! `duplicate_table_aliases`: SQLCLEAN009's aliases given twice in one scope.

use crate::models::{RuleFinding, RuleInput};
use crate::names::main::repeated_names::repeated_names;

/// The relation aliases that repeat an earlier alias of their scope.
pub fn duplicate_table_aliases(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        for scope in statement.query_scopes().scopes {
            let aliases = scope.relations.iter().filter_map(|relation| relation.alias);
            found.extend(repeated_names(&input.dialect, aliases));
        }
    }
    found
}
