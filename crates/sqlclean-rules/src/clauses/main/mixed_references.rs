//! `mixed_references`: SQLCLEAN011's clauses that mix positions and names.

use sqlquack_syntax::Expression;

use crate::clauses::_helpers::orders::query_order;
use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::number_text::number_text;
use crate::trees::main::statement_nodes::statement_nodes;

/// The GROUP BY and ORDER BY keywords of clauses with a position beside a name or expression.
pub fn mixed_references(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for node in statement_nodes(input) {
        if let Expression::Select(select) = node
            && let Some(group) = &select.group_by
            && is_mixed(group.expressions.iter())
        {
            found.extend(
                group
                    .keyword_span
                    .0
                    .map(|span| RuleFinding { span, fix: None }),
            );
        }
        if let Some(order) = query_order(node)
            && is_mixed(order.expressions.iter().map(|item| &item.this))
        {
            found.extend(
                order
                    .keyword_span
                    .0
                    .map(|span| RuleFinding { span, fix: None }),
            );
        }
    }
    found
}

fn is_mixed<'a>(items: impl Iterator<Item = &'a Expression>) -> bool {
    let (mut position, mut name): (bool, bool) = (false, false);
    for item in items {
        let numbered: bool = number_text(item).is_some();
        position |= numbered;
        name |= !numbered;
    }
    position && name
}
