//! `implicit_joins`: SQLCLEAN002's comma joins.

use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::joins::_helpers::written_joins::has_explicit_join;
use crate::models::{Replacement, RuleEdit, RuleFinding, RuleInput};

/// The commas that join relations, without those before an item that may read the items before it.
pub fn implicit_joins(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for select in input
        .statements
        .iter()
        .flat_map(|statement| statement.dfs())
        .filter_map(Expression::as_select)
    {
        let items = select.items_after_commas();
        let mixed: bool = !items.is_empty() && has_explicit_join(select);
        for item in items {
            let Some(comma) = item.comma.0 else { continue };
            if item.may_read_earlier() {
                continue;
            }
            let fix: Option<RuleEdit> = (!mixed).then_some(RuleEdit {
                span: comma,
                replacement: Replacement::Text(" CROSS JOIN "),
                trailing_layout: true,
            });
            found.push(RuleFinding { span: comma, fix });
        }
    }
    found
}
