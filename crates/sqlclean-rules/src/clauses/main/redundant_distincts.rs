//! `redundant_distincts`: SQLCLEAN006's DISTINCT over groups the select list already shows.

use crate::clauses::_helpers::groups::is_redundant;
use crate::models::{Replacement, RuleEdit, RuleFinding, RuleInput};
use crate::trees::main::selects::selects;

/// The DISTINCTs whose GROUP BY keys are all projected, with their removal as the fix.
pub fn redundant_distincts(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    selects(input)
        .filter(|select| is_redundant(select))
        .filter_map(|select| select.distinct_span.0)
        .map(|span| RuleFinding {
            span,
            fix: Some(RuleEdit {
                span,
                replacement: Replacement::Text(""),
                trailing_layout: true,
            }),
        })
        .collect()
}
