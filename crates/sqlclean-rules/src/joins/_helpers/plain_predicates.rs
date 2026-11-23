//! Join conditions that only compare plain operands.

use sqlquack_syntax::Expression;

use crate::models::DialectFacts;

/// Whether ON `condition` compares plain columns and literals alone.
pub(crate) fn is_plain_condition(condition: &Expression, dialect: &DialectFacts) -> bool {
    plain_predicate(condition) || (dialect.negated_distinct_from && is_distinct_from(condition))
}

fn plain_column(expression: &Expression) -> bool {
    matches!(
        expression,
        Expression::Column(_) | Expression::Identifier(_)
    ) || matches!(expression, Expression::Paren(value) if plain_column(&value.this))
}

fn literal(expression: &Expression) -> bool {
    matches!(
        expression,
        Expression::Literal(_) | Expression::Boolean(_) | Expression::Null(_)
    ) || matches!(expression, Expression::Neg(value) if matches!(&value.this, Expression::Literal(_)))
}

fn operand(expression: &Expression) -> bool {
    plain_column(expression) || literal(expression)
}

fn comparison(left: &Expression, right: &Expression) -> bool {
    operand(left) && operand(right)
}

/// `a IS DISTINCT FROM b`, which some parsers write as NOT over `IS NOT DISTINCT FROM`.
fn is_distinct_from(expression: &Expression) -> bool {
    matches!(expression, Expression::Not(value)
        if matches!(value.this, Expression::NullSafeEq(_)) && plain_predicate(&value.this))
}

/// Whether every comparison joined by AND, OR and parentheses compares plain operands, read without recursion.
fn plain_predicate(expression: &Expression) -> bool {
    let mut pending: Vec<&Expression> = vec![expression];
    while let Some(node) = pending.pop() {
        match node {
            Expression::Paren(value) => pending.push(&value.this),
            Expression::Annotated(value) => pending.push(&value.this),
            Expression::And(value) | Expression::Or(value) => {
                pending.extend([&value.right, &value.left]);
            }
            _ if !plain_comparison(node) => return false,
            _ => {}
        }
    }
    true
}

fn plain_comparison(expression: &Expression) -> bool {
    match expression {
        Expression::Boolean(_) => true,
        Expression::Eq(value)
        | Expression::Neq(value)
        | Expression::Lt(value)
        | Expression::Lte(value)
        | Expression::Gt(value)
        | Expression::Gte(value)
        | Expression::NullSafeEq(value)
        | Expression::NullSafeNeq(value)
        | Expression::Adjacent(value)
        | Expression::ArrayContainsAll(value)
        | Expression::ArrayContainedBy(value)
        | Expression::ArrayOverlaps(value)
        | Expression::ExtendsLeft(value)
        | Expression::ExtendsRight(value)
        | Expression::Match(value)
        | Expression::TsMatch(value) => comparison(&value.left, &value.right),
        Expression::Like(value) | Expression::ILike(value) => {
            comparison(&value.left, &value.right)
                && value.escape.as_ref().is_none_or(literal)
                && value.quantifier.is_none()
        }
        Expression::Between(value) => {
            plain_column(&value.this) && operand(&value.low) && operand(&value.high)
        }
        Expression::In(value) => {
            plain_column(&value.this)
                && value.query.is_none()
                && value.unnest.is_none()
                && !value.is_field
                && !value.expressions.is_empty()
                && value.expressions.iter().all(literal)
        }
        Expression::IsNull(value) => plain_column(&value.this),
        _ => false,
    }
}
