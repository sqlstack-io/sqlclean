//! `ranking_orders`: SQLCLEAN018's undetermined rankings and SQLCLEAN043's long sorts.

use sqlquack_syntax::Expression;

use crate::models::{RankingFinding, RuleInput};
use crate::rankings::_helpers::keys::{covers, key_environment, window_columns};
use crate::rankings::_helpers::ranking::{effective_over, ranking_windows};
use crate::rankings::constants::ROW_NUMBER;
use crate::rankings::constants::TIE_SENSITIVE_FUNCTIONS;
use crate::trees::main::call_positions::call_positions;

/// The ranking windows with no order, an order not proven unique, or more sort keys than allowed.
pub fn ranking_orders(input: &RuleInput<'_>) -> Vec<RankingFinding> {
    let environment = key_environment(input);
    let mut found: Vec<RankingFinding> = Vec::new();
    for ranking in ranking_windows(input.statements) {
        let Some(over) = effective_over(&ranking.window.over, &ranking.windows) else {
            continue;
        };
        let order: Vec<&Expression> = over.order_by.iter().map(|ordered| &ordered.this).collect();
        let exceeds_cap: bool = order.len() > input.settings.max_ranking_order_by;
        let unordered: bool = order.is_empty() && ranking.function == ROW_NUMBER;
        let unproven: bool = !order.is_empty()
            && TIE_SENSITIVE_FUNCTIONS.contains(&ranking.function)
            && !covers(
                &environment.input_keys(ranking.select),
                &window_columns(&over.partition_by, &order),
            );
        if !(exceeds_cap || unordered || unproven) {
            continue;
        }
        if let Some(span) = call_positions(ranking.called).0.0.or(ranking.window.span.0) {
            found.push(RankingFinding {
                span,
                sort_keys: order.len(),
                exceeds_cap,
                unordered,
                unproven,
            });
        }
    }
    found
}
