//! `unused_cte_outputs`: SQLCLEAN042's columns no later step reads.

use crate::models::{OutputFinding, RuleInput};
use crate::outputs::_helpers::cte_columns::unused_columns;

/// The unread output columns of non-final CTEs, outside fixture bodies.
pub fn unused_cte_outputs(input: &RuleInput<'_>) -> Vec<OutputFinding> {
    match input.settings.fixture {
        Some(_) => Vec::new(),
        None => unused_columns(input),
    }
}
