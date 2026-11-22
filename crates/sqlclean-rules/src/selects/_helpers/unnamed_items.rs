//! Select items that give their output no name.

use sqlquack_syntax::expressions::queries::models::Select;
use sqlquack_syntax::tokens::models::OperatorSpan;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{RuleFinding, RuleInput};
use crate::selects::_helpers::stars::is_wildcard;
use crate::trees::main::is_generated::is_generated;
use crate::trees::main::number_text::number_text;

const CEREMONIAL_SELECT: &str = "1";

/// The first word of each unnamed calculation of `select`.
pub(crate) fn unnamed_items(input: &RuleInput<'_>, select: &Select) -> Vec<RuleFinding> {
    if input.settings.fixture.is_some()
        && let [item] = select.expressions.as_slice()
        && number_text(item) == Some(CEREMONIAL_SELECT)
    {
        return Vec::new();
    }
    let mut found: Vec<RuleFinding> = Vec::new();
    for (index, (item, written)) in select
        .expressions
        .iter()
        .zip(&select.item_spans.0)
        .enumerate()
    {
        if is_named(item) || is_one_word(item, written) || holds_placeholder(input, item) {
            continue;
        }
        let head = match index {
            0 => select.list_head_span.0.or(written.operator()),
            _ => written.operator(),
        };
        found.extend(head.map(|span| RuleFinding { span, fix: None }));
    }
    found
}

fn is_named(item: &Expression) -> bool {
    let mut node: &Expression = item;
    loop {
        node = match node {
            Expression::Alias(_) | Expression::Column(_) | Expression::Identifier(_) => {
                return true;
            }
            Expression::Annotated(annotated) => &annotated.this,
            Expression::Dot(dot) => &dot.this,
            _ => return is_wildcard(node),
        };
    }
}

/// Whether the item is one word that names itself, as CURRENT_DATE does.
fn is_one_word(item: &Expression, written: &OperatorSpan) -> bool {
    if matches!(
        item,
        Expression::Literal(_) | Expression::Boolean(_) | Expression::Null(_)
    ) {
        return false;
    }
    match (written.operator(), written.operation()) {
        (Some(head), Some(whole)) => head.end == whole.end,
        _ => false,
    }
}

fn holds_placeholder(input: &RuleInput<'_>, item: &Expression) -> bool {
    item.dfs().any(|node| match node {
        Expression::Column(column) => {
            is_generated(input, &column.name.name)
                || column
                    .table
                    .as_ref()
                    .is_some_and(|table| is_generated(input, &table.name))
        }
        Expression::Identifier(identifier) => is_generated(input, &identifier.name),
        Expression::Table(table) => is_generated(input, &table.name.name),
        Expression::Function(call) => is_generated(input, &call.name),
        Expression::Var(var) => is_generated(input, &var.this),
        _ => false,
    })
}
