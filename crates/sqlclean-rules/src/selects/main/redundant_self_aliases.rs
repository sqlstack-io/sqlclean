//! `redundant_self_aliases`: SQLCLEAN016's columns aliased to their own name.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::names::models::Identifier;
use sqlquack_syntax::tokens::models::Span;

use crate::models::{RuleFinding, RuleInput};
use crate::names::main::resolved::resolved;
use crate::trees::main::selects::selects;

/// The `AS name` after a column of the same resolved name, from the end of the column.
pub fn redundant_self_aliases(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    selects(input)
        .flat_map(|select| &select.expressions)
        .filter_map(|item| self_alias(input, item))
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}

fn self_alias(input: &RuleInput<'_>, item: &Expression) -> Option<Span> {
    let Expression::Alias(alias) = item else {
        return None;
    };
    if !alias.alias_explicit_as {
        return None;
    }
    let (name, written): (&Identifier, Option<Span>) = match &alias.this {
        Expression::Column(column) => (&column.name, column.span.or(column.name.span)),
        Expression::Dot(dot) if is_dotted_name(&dot.this) => (&dot.field, dot.field.span),
        _ => return None,
    };
    let (column, named): (Span, Span) = (written?, alias.alias.span?);
    let same: bool = resolved(&input.dialect, name) == resolved(&input.dialect, &alias.alias);
    same.then(|| Span::new(column.end, named.end, named.line, named.column))
}

fn is_dotted_name(mut receiver: &Expression) -> bool {
    loop {
        receiver = match receiver {
            Expression::Column(_) | Expression::Identifier(_) => return true,
            Expression::Dot(dot) => &dot.this,
            _ => return false,
        };
    }
}
