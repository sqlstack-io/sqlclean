//! SQLCLEAN002 and SQLCLEAN003 on the located tree: comma joins, and joins without a condition.

use sqlquack_syntax::expressions::queries::models::{Join, JoinKind, Select};
use sqlquack_syntax::tokens::models::Token;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::lint::models::{AnchoredFinding, LintEdit};

/// The commas that join relations, without those before an item that may read the items before it.
pub(super) fn implicit_joins(
    statements: &[Expression],
    tokens: &[Token],
    sql: &str,
) -> Vec<AnchoredFinding> {
    let mut found: Vec<AnchoredFinding> = Vec::new();
    for select in statements
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
            let fix: Option<LintEdit> = match mixed {
                true => None,
                false => cross_join_edit(tokens, sql, comma.start, comma.end),
            };
            found.push(AnchoredFinding { span: comma, fix });
        }
    }
    found
}

/// Whether the FROM clause writes a join, which binds tighter than its commas.
fn has_explicit_join(select: &Select) -> bool {
    !select.joins.is_empty()
        || select
            .from
            .iter()
            .flat_map(|from| &from.expressions)
            .any(writes_a_join)
}

/// Whether a join is written anywhere in FROM item `item`.
fn writes_a_join(item: &Expression) -> bool {
    item.dfs()
        .flat_map(Expression::own_joins)
        .any(|join| join.kind != JoinKind::Implicit)
}

/// The edit that writes the comma at `start..end` as CROSS JOIN, up to the text after it.
fn cross_join_edit(tokens: &[Token], sql: &str, start: usize, end: usize) -> Option<LintEdit> {
    let next: usize = crate::lint::_helpers::engine::next_text_start(tokens, end)?;
    let replaced: &str = crate::lint::_helpers::engine::char_slice(sql, start, next)?;
    if crate::lint::_helpers::engine::contains_comment(replaced) {
        return None;
    }
    Some(LintEdit {
        start,
        end: next,
        replacement: " CROSS JOIN ".to_owned(),
    })
}

/// The joins that need a condition and have none, or only one that is always true.
pub(super) fn joins_without_condition(statements: &[Expression]) -> Vec<AnchoredFinding> {
    statements
        .iter()
        .flat_map(|statement| statement.dfs())
        .flat_map(Expression::own_joins)
        .filter(|join| lacks_condition(join))
        .filter_map(conditionless_finding)
        .collect()
}

/// The finding on `join`'s JOIN keyword, with CROSS JOIN as its fix when JOIN is written alone.
fn conditionless_finding(join: &Join) -> Option<AnchoredFinding> {
    let span = join.keyword_span.0?;
    let fix: Option<LintEdit> = bare_join(join).then(|| LintEdit {
        start: span.start,
        end: span.end,
        replacement: "CROSS JOIN".to_owned(),
    });
    Some(AnchoredFinding { span, fix })
}

/// Whether `join` takes a condition and has no meaningful one.
fn lacks_condition(join: &Join) -> bool {
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

/// `expression` without the parentheses and comments around it.
fn unwrapped(mut expression: &Expression) -> &Expression {
    loop {
        expression = match expression {
            Expression::Paren(node) => &node.this,
            Expression::Annotated(node) => &node.this,
            _ => return expression,
        };
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
