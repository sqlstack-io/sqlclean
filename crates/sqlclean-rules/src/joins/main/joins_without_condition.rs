//! `joins_without_condition`: SQLCLEAN003's joins.

use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::joins::_helpers::conditions::{conditionless_finding, lacks_condition};
use crate::models::{RuleFinding, RuleInput};

/// The joins that need a condition and have none, or only one that is always true.
pub fn joins_without_condition(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    input
        .statements
        .iter()
        .flat_map(|statement| statement.dfs())
        .flat_map(Expression::own_joins)
        .filter(|join| lacks_condition(join))
        .filter_map(conditionless_finding)
        .collect()
}
