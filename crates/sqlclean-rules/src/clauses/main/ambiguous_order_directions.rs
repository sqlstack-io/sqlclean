//! `ambiguous_order_directions`: SQLCLEAN026's ORDER BYs that state only some directions.

use crate::clauses::_helpers::orders::{query_order, states_some_directions};
use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::statement_nodes::statement_nodes;

/// The ORDER keywords of lists with ASC or DESC on some items and neither on others.
pub fn ambiguous_order_directions(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .filter_map(query_order)
        .filter(|order| states_some_directions(order))
        .filter_map(|order| order.keyword_span.0)
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}
