//! `computed_conditions`: SQLCLEAN040's computed join conditions.

use crate::joins::_helpers::plain_predicates::is_plain_condition;
use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::node_joins::node_joins;
use crate::trees::main::statement_nodes::statement_nodes;

/// The ON of each join whose condition is more than comparisons of plain columns and literals.
pub fn computed_conditions(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .flat_map(node_joins)
        .filter_map(|join| {
            let condition = join.on.as_ref()?;
            let span = join.on_span.0?;
            (!is_plain_condition(condition, &input.dialect))
                .then_some(RuleFinding { span, fix: None })
        })
        .collect()
}
