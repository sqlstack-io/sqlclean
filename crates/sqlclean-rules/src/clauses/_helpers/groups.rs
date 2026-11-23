//! Whether groups are already the output.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::Select;

pub(crate) fn is_redundant(select: &Select) -> bool {
    let Some(group) = &select.group_by else {
        return false;
    };
    select.distinct
        && select.distinct_on.is_none()
        && !group.totals
        && !group.with_modifier
        && !group.expressions.is_empty()
        && group
            .expressions
            .iter()
            .all(|key| is_projected(key, &select.expressions))
}

fn is_projected(key: &Expression, items: &[Expression]) -> bool {
    items.iter().any(|item| is_same_value(key, item))
}

fn is_same_value(key: &Expression, item: &Expression) -> bool {
    let value: &Expression = match item {
        Expression::Alias(alias) => &alias.this,
        other => other,
    };
    if key == value {
        return true;
    }
    let (Expression::Column(key), Expression::Column(value)) = (key, value) else {
        return false;
    };
    let same_table: bool = match (&key.table, &value.table) {
        (Some(left), Some(right)) => left.name.eq_ignore_ascii_case(&right.name),
        (left, right) => left.is_none() && right.is_none(),
    };
    same_table && key.name.name.eq_ignore_ascii_case(&value.name.name)
}
