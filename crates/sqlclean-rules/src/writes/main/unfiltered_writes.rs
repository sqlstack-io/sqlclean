//! `unfiltered_writes`: SQLCLEAN049's writes that change every row.

use sqlquack_syntax::Expression;

use crate::models::{RuleFinding, RuleInput};
use crate::writes::_helpers::limits::{delete_is_limited, update_is_limited};

/// The UPDATE and DELETE statements with nothing that limits the rows they change.
pub fn unfiltered_writes(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    input
        .statements
        .iter()
        .filter_map(|statement| match statement {
            Expression::Update(update) if !update_is_limited(update) => update.keyword_span.0,
            Expression::Delete(delete) if !delete_is_limited(delete) => delete.keyword_span.0,
            _ => None,
        })
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}
