//! `duplicate_cte_names`: SQLCLEAN046's names given twice in one WITH.

use sqlquack_syntax::Expression;

use crate::models::{RuleFinding, RuleInput};
use crate::names::main::repeated_names::repeated_names;
use crate::trees::main::statement_nodes::statement_nodes;

/// The CTE names the dialect takes for an earlier one of their WITH.
pub fn duplicate_cte_names(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .filter_map(Expression::with_clause)
        .flat_map(|with| repeated_names(&input.dialect, with.ctes.iter().map(|cte| &cte.alias)))
        .collect()
}
