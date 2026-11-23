//! Which joins need a condition and have none.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::{Join, JoinKind};

use crate::models::{Replacement, RuleEdit, RuleFinding};
use crate::trees::main::unwrapped::unwrapped;

/// The finding on `join`'s JOIN keyword, with CROSS JOIN as its fix when JOIN is written alone.
pub(crate) fn conditionless_finding(join: &Join) -> Option<RuleFinding> {
    let span = join.keyword_span.0?;
    let fix: Option<RuleEdit> = bare_join(join).then_some(RuleEdit {
        span,
        replacement: Replacement::Text("CROSS JOIN"),
        trailing_layout: false,
    });
    Some(RuleFinding { span, fix })
}

/// Whether `join` takes a condition and has no meaningful one.
pub(crate) fn lacks_condition(join: &Join) -> bool {
    if !takes_condition(join.kind)
        || !join.using.is_empty()
        || join.match_condition.is_some()
        || join.nearest_by.is_some()
        || join.may_read_earlier()
    {
        return false;
    }
    join.on.as_ref().is_none_or(is_always_true)
}

/// Whether a join of `kind` is written with a condition.
fn takes_condition(kind: JoinKind) -> bool {
    !matches!(
        kind,
        JoinKind::Cross
            | JoinKind::CrossApply
            | JoinKind::OuterApply
            | JoinKind::Implicit
            | JoinKind::Positional
            | JoinKind::Paste
            | JoinKind::Array
            | JoinKind::LeftArray
            | JoinKind::Natural
            | JoinKind::NaturalLeft
            | JoinKind::NaturalRight
            | JoinKind::NaturalFull
            | JoinKind::NaturalSemi
            | JoinKind::NaturalAnti
    )
}

/// Whether a predicate is TRUE, or one literal compared equal to itself.
fn is_always_true(predicate: &Expression) -> bool {
    match unwrapped(predicate) {
        Expression::Boolean(boolean) => boolean.value,
        Expression::Eq(comparison) => {
            match (unwrapped(&comparison.left), unwrapped(&comparison.right)) {
                (Expression::Literal(left), Expression::Literal(right)) => left == right,
                (Expression::Boolean(left), Expression::Boolean(right)) => {
                    left.value == right.value
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Whether `join` is written as JOIN alone, which CROSS JOIN can replace.
fn bare_join(join: &Join) -> bool {
    join.kind == JoinKind::Inner
        && !join.use_inner_keyword
        && !join.directed
        && join.join_hint.is_none()
        && join.on.is_none()
}
