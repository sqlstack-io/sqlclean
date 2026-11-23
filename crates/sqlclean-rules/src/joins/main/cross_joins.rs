//! `cross_joins`: SQLCLEAN038's cross joins that can multiply rows.

use crate::joins::_helpers::cross_targets::{cross_targets, joined_relation_name};
use crate::joins::_helpers::one_row::one_row;
use crate::models::{CrossJoinFinding, RuleInput};

/// The CROSS JOINs where neither side provably holds at most one row.
pub fn cross_joins<'a>(input: &RuleInput<'a>) -> Vec<CrossJoinFinding<'a>> {
    let known = one_row(input);
    let mut found: Vec<CrossJoinFinding<'a>> = Vec::new();
    for target in input.statements.iter().flat_map(cross_targets) {
        let Some(span) = target.join.kind_span.0 else {
            continue;
        };
        let single: bool = known.relation_one_row(&target.join.this)
            || target
                .first
                .is_some_and(|first| known.relation_one_row(first));
        if !single {
            found.push(CrossJoinFinding {
                span,
                relation: joined_relation_name(&target.join.this),
                cte: target.cte,
            });
        }
    }
    found
}
