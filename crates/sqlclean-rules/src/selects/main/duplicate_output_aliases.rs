//! `duplicate_output_aliases`: SQLCLEAN010's output names given twice.

use sqlquack_syntax::Expression;

use crate::models::{RuleFinding, RuleInput};
use crate::names::main::repeated_names::repeated_names;
use crate::trees::main::selects::selects;

/// The aliases that repeat an earlier alias of their select list.
pub fn duplicate_output_aliases(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    selects(input)
        .flat_map(|select| {
            let aliases = select.expressions.iter().filter_map(|item| match item {
                Expression::Alias(alias) => Some(&alias.alias),
                _ => None,
            });
            repeated_names(&input.dialect, aliases)
        })
        .collect()
}
