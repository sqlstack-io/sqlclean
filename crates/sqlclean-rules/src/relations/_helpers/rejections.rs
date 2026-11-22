//! Predicates that drop the NULL rows of a LEFT JOIN.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::{Join, JoinKind};

use crate::trees::main::unwrapped::unwrapped;

/// Whether `join` keeps left rows with NULLs for the right side.
pub(crate) fn is_left(join: &Join) -> bool {
    matches!(
        join.kind,
        JoinKind::Left | JoinKind::NaturalLeft | JoinKind::AsOfLeft | JoinKind::LeftLateral
    )
}

/// The qualifiers of columns that `condition` cannot be true for when NULL; none when it holds OR.
pub(crate) fn rejected_qualifiers(condition: &Expression) -> Vec<&str> {
    let mut found: Vec<&str> = Vec::new();
    let mut pending: Vec<&Expression> = vec![condition];
    while let Some(node) = pending.pop() {
        match unwrapped(node) {
            Expression::Or(_) => return Vec::new(),
            Expression::And(operation) => pending.extend([&operation.left, &operation.right]),
            tested => found.extend(tested_column(tested).and_then(qualifier)),
        }
    }
    found
}

fn tested_column(test: &Expression) -> Option<&Expression> {
    match test {
        Expression::Eq(node)
        | Expression::Neq(node)
        | Expression::Lt(node)
        | Expression::Lte(node)
        | Expression::Gt(node)
        | Expression::Gte(node) => Some(&node.left),
        Expression::Like(node) | Expression::ILike(node) if !node.negated => Some(&node.left),
        Expression::In(node) if !node.not => Some(&node.this),
        Expression::Between(node) if !node.not => Some(&node.this),
        Expression::IsNull(node) if node.not => Some(&node.this),
        Expression::Not(node) => match unwrapped(&node.this) {
            Expression::IsNull(inner) if !inner.not => Some(&inner.this),
            _ => None,
        },
        _ => None,
    }
}

fn qualifier(tested: &Expression) -> Option<&str> {
    match tested {
        Expression::Column(column) => column.table.as_ref().map(|table| table.name.as_str()),
        _ => None,
    }
}
