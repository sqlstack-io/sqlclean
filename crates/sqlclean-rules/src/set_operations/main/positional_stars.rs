//! `positional_stars`: SQLCLEAN007's wildcards paired by position.

use crate::models::{RuleFinding, RuleInput};
use crate::set_operations::_helpers::operations::positional_operations;

/// The operators that pair a projected wildcard by position.
pub fn positional_stars(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    positional_operations(input, |set| {
        set.left.last_branch().projects_wildcard() || set.right.first_branch().projects_wildcard()
    })
}
