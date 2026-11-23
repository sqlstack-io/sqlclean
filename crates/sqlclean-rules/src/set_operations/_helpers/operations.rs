//! The branches either side of a set operator.

use sqlquack_syntax::traversal::models::SetOperation;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{RuleFinding, RuleInput};

/// The operators that pair by position and satisfy `reported`.
pub(crate) fn positional_operations(
    input: &RuleInput<'_>,
    reported: impl Fn(&SetOperation<'_>) -> bool,
) -> Vec<RuleFinding> {
    input
        .statements
        .iter()
        .flat_map(|statement| statement.dfs())
        .filter_map(Expression::set_operation)
        .filter(|set| !set.by_name && reported(set))
        .filter_map(|set| set.operator_span.0)
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}

/// The width of the branch next to an operator; none past a pairing by name, which a schema decides.
pub(crate) fn adjacent_width(side: &Expression, last: bool) -> Option<usize> {
    let mut query: &Expression = side;
    loop {
        query = match query {
            Expression::Annotated(node) => &node.this,
            Expression::Paren(node) => &node.this,
            Expression::Subquery(node) => &node.this,
            _ => match query.set_operation() {
                Some(set) if set.by_name => return None,
                Some(set) if last => set.right,
                Some(set) => set.left,
                None => return query.column_count(),
            },
        };
    }
}
