//! `statement_nodes`: every node of the statements a Rule is handed.

use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::RuleInput;

/// Every node of every statement, each statement depth first.
pub(crate) fn statement_nodes<'a>(input: &RuleInput<'a>) -> impl Iterator<Item = &'a Expression> {
    input
        .statements
        .iter()
        .flat_map(|statement| statement.dfs())
}
