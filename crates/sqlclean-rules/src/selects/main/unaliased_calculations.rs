//! `unaliased_calculations`: SQLCLEAN022's outputs without a stable name.

use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{RuleFinding, RuleInput};
use crate::selects::_helpers::unnamed_items::{column_aliased_body, unnamed_items};
use crate::trees::main::placed_queries::placed_queries;

/// The calculations without an alias in queries whose output names are their first branch's.
pub fn unaliased_calculations(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        let mut listed: Vec<&Expression> = statement
            .dfs()
            .filter_map(Expression::with_clause)
            .flat_map(|with| &with.ctes)
            .filter(|cte| !cte.columns.is_empty())
            .map(|cte| cte.this.first_branch())
            .collect();
        listed.extend(statement.dfs().filter_map(column_aliased_body));
        for query in placed_queries(statement) {
            let named_by: &Expression = query.first_branch();
            if listed.iter().any(|body| std::ptr::eq(*body, named_by)) {
                continue;
            }
            if let Expression::Select(select) = named_by {
                found.extend(unnamed_items(input, select));
            }
        }
    }
    found
}
