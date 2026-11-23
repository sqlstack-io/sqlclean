//! `duplicate_cte_names`: SQLCLEAN046's names given twice in one WITH.

use sqlquack_syntax::Expression;

use crate::models::{RuleFinding, RuleInput};
use crate::names::main::repeated_names::repeated_names;
use crate::trees::main::statement_nodes::statement_nodes;

/// The CTE names the dialect takes for an earlier one of their WITH.
pub fn duplicate_cte_names(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for with in statement_nodes(input).filter_map(Expression::with_clause) {
        let names = with.ctes.iter().map(|cte| &cte.alias);
        found.extend(repeated_names(&input.dialect, names));
    }
    found
}
