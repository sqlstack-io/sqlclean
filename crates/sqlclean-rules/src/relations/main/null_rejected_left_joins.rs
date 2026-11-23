//! `null_rejected_left_joins`: SQLCLEAN024's LEFT JOINs undone by WHERE.

use crate::models::{RuleFinding, RuleInput};
use crate::relations::_helpers::references::joined_name;
use crate::relations::_helpers::rejections::{is_left, rejected_qualifiers};
use crate::trees::main::join_chains::join_chains;
use crate::trees::main::selects::selects;

/// The LEFT of each join whose right side a WHERE without OR rejects the NULLs of.
pub fn null_rejected_left_joins(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for select in selects(input) {
        let Some(condition) = &select.where_clause else {
            continue;
        };
        let rejected: Vec<&str> = rejected_qualifiers(&condition.this);
        if rejected.is_empty() {
            continue;
        }
        for join in join_chains(select).into_iter().flat_map(|(_, joins)| joins) {
            let undone: bool = is_left(join)
                && joined_name(&join.this).is_some_and(|name| {
                    rejected.iter().any(|used| used.eq_ignore_ascii_case(name))
                });
            if undone {
                found.extend(join.kind_span.0.map(|span| RuleFinding { span, fix: None }));
            }
        }
    }
    found
}
