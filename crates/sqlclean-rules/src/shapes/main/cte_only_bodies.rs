//! `cte_only_bodies`: SQLCLEAN034's logic outside top-level CTEs.

use crate::models::{RuleFinding, RuleInput};
use crate::shapes::_helpers::terminals::{has_logic, terminal};

/// The terminal set operator, or the terminal SELECT of a body without CTEs or with logic there.
pub fn cte_only_bodies(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let Some(step) = terminal(input) else {
        return Vec::new();
    };
    let span = match (step.operator, step.select) {
        (Some(operator), _) => Some(operator),
        (None, Some(select)) if step.with.is_none() || has_logic(select) => select.select_span.0,
        _ => None,
    };
    span.map(|span| RuleFinding { span, fix: None })
        .into_iter()
        .collect()
}
