//! DISTINCT written before parentheses.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::Select;
use sqlquack_syntax::tokens::models::Span;

use crate::models::{Replacement, RuleEdit, RuleFinding};

/// The finding of a SELECT DISTINCT whose first item begins with parentheses.
pub(crate) fn select_distinct(select: &Select) -> Option<RuleFinding> {
    if !select.distinct || select.distinct_on.is_some() {
        return None;
    }
    let distinct: Span = select.distinct_span.0?;
    let item: &Expression = select.expressions.first()?;
    let value: &Expression = match item {
        Expression::Alias(alias) => &alias.this,
        other => other,
    };
    let written = select.item_spans.0.first()?;
    if let Expression::Tuple(_) = value {
        let parenthesised: bool = written
            .operator()
            .is_some_and(|head| head.end == head.start + 1);
        let whole: Span = distinct.through(written.operation()?);
        return parenthesised.then_some(RuleFinding {
            span: whole,
            fix: None,
        });
    }
    let opened: usize = written.operator()?.start;
    parenthesised_finding(distinct, value).filter(|_| begins_at(value, opened))
}

/// The finding of a call's DISTINCT whose first argument begins with parentheses.
pub(crate) fn call_distinct(call: &Expression) -> Option<RuleFinding> {
    let (arguments, distinct, marked) = match call {
        Expression::Function(call) => (
            &call.args,
            call.set_quantifier.is_distinct(),
            call.argument_span,
        ),
        Expression::AggregateFunction(call) => (
            &call.args,
            call.set_quantifier.is_distinct(),
            call.argument_span,
        ),
        _ => return None,
    };
    let keyword: Span = marked.0.filter(|_| distinct)?;
    parenthesised_finding(keyword, arguments.first()?)
}

/// `DISTINCT (..)`, with a fix only where the parentheses hold a whole value that is no query.
fn parenthesised_finding(keyword: Span, value: &Expression) -> Option<RuleFinding> {
    let leading: &Expression = leading_operand(value);
    let group: Span = match leading {
        Expression::Paren(paren) => paren.span.0?,
        Expression::Subquery(query) => query.span.0?,
        _ => return None,
    };
    let span: Span = keyword.through(group);
    let whole: bool = std::ptr::eq(leading, value);
    let fix: Option<RuleEdit> = match leading {
        Expression::Paren(paren)
            if whole
                && !matches!(
                    paren.this,
                    Expression::Select(_)
                        | Expression::Subquery(_)
                        | Expression::Tuple(_)
                        | Expression::Union(_)
                        | Expression::Intersect(_)
                        | Expression::Except(_)
                ) =>
        {
            Some(RuleEdit {
                span,
                replacement: Replacement::Wrapped {
                    before: "DISTINCT ",
                    kept: Span {
                        start: group.start + 1,
                        end: group.end - 1,
                        ..group
                    },
                    after: "",
                },
                trailing_layout: false,
            })
        }
        _ => None,
    };
    Some(RuleFinding { span, fix })
}

fn begins_at(value: &Expression, opened: usize) -> bool {
    match leading_operand(value) {
        Expression::Paren(paren) => paren.span.0.is_some_and(|span| span.start == opened),
        Expression::Subquery(query) => query.span.0.is_some_and(|span| span.start == opened),
        _ => false,
    }
}

/// The operand written first in `value`.
fn leading_operand(mut value: &Expression) -> &Expression {
    loop {
        value = match value {
            Expression::Annotated(node) => &node.this,
            Expression::Cast(node) if node.double_colon_syntax => &node.this,
            Expression::In(node) => &node.this,
            Expression::Between(node) => &node.this,
            Expression::IsNull(node) => &node.this,
            Expression::Like(node) | Expression::ILike(node) => &node.left,
            Expression::Dot(node) => &node.this,
            Expression::Subscript(node) => &node.this,
            Expression::Operator(node) => &node.this,
            Expression::Add(node)
            | Expression::Sub(node)
            | Expression::Mul(node)
            | Expression::Div(node)
            | Expression::Mod(node)
            | Expression::Concat(node)
            | Expression::And(node)
            | Expression::Or(node)
            | Expression::Eq(node)
            | Expression::Neq(node)
            | Expression::Lt(node)
            | Expression::Lte(node)
            | Expression::Gt(node)
            | Expression::Gte(node)
            | Expression::Is(node)
            | Expression::BitwiseAnd(node)
            | Expression::BitwiseOr(node)
            | Expression::BitwiseXor(node)
            | Expression::BitwiseLeftShift(node)
            | Expression::BitwiseRightShift(node) => &node.left,
            _ => return value,
        };
    }
}
