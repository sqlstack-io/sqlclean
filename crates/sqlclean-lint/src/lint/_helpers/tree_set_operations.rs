//! SQLCLEAN007 and SQLCLEAN020 on the located tree: the branches either side of a set operator.

use sqlquack_syntax::tokens::models::Span;
use sqlquack_syntax::traversal::models::SetOperation;
use sqlquack_syntax::{Expression, ExpressionWalk};

/// The operators that pair a projected wildcard by position.
pub(super) fn positional_stars(statements: &[Expression]) -> Vec<Span> {
    positional_operations(statements, |set| {
        set.left.last_branch().projects_wildcard() || set.right.first_branch().projects_wildcard()
    })
}

/// The operators whose neighbouring branches have different column counts.
pub(super) fn arity_mismatches(statements: &[Expression]) -> Vec<Span> {
    positional_operations(statements, |set| {
        adjacent_width(set.left, true)
            .zip(adjacent_width(set.right, false))
            .is_some_and(|(left, right)| left != right)
    })
}

/// The operators that pair by position and satisfy `reported`.
fn positional_operations(
    statements: &[Expression],
    reported: impl Fn(&SetOperation<'_>) -> bool,
) -> Vec<Span> {
    statements
        .iter()
        .flat_map(|statement| statement.dfs())
        .filter_map(Expression::set_operation)
        .filter(|set| !set.by_name && reported(set))
        .filter_map(|set| set.operator_span.0)
        .collect()
}

/// The width of the branch next to an operator; none past a pairing by name, which a schema decides.
fn adjacent_width(side: &Expression, last: bool) -> Option<usize> {
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
