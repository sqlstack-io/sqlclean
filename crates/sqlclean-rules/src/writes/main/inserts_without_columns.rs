//! `inserts_without_columns`: SQLCLEAN050's inserts that leave their columns to the table.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::statements::models::Insert;

use crate::models::{RuleFinding, RuleInput};

/// The INSERT statements without a column list, DEFAULT VALUES or BY NAME.
pub fn inserts_without_columns(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    input
        .statements
        .iter()
        .filter_map(|statement| match statement {
            Expression::Insert(insert) if lacks_column_list(insert) => insert.keyword_span.0,
            _ => None,
        })
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}

fn lacks_column_list(insert: &Insert) -> bool {
    insert.columns.is_empty()
        && !insert.default_values
        && !insert.by_name
        && insert.directory.is_none()
        && (insert.query.is_some() || !insert.values.is_empty())
}
