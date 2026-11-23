//! `arity_mismatches`: SQLCLEAN020's branches of different widths.

use crate::models::{RuleFinding, RuleInput};
use crate::set_operations::_helpers::operations::{adjacent_width, positional_operations};

/// The operators whose neighbouring branches have different column counts.
pub fn arity_mismatches(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    positional_operations(input, |set| {
        adjacent_width(set.left, true)
            .zip(adjacent_width(set.right, false))
            .is_some_and(|(left, right)| left != right)
    })
}
