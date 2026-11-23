//! Wildcards in a select list, and where their asterisk is written.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::names::models::Star;
use sqlquack_syntax::tokens::models::Span;

use crate::trees::main::unwrapped::unwrapped;

const WILDCARD: &str = "*";

/// Whether select item `item` is a wildcard.
pub(crate) fn is_wildcard(item: &Expression) -> bool {
    match unwrapped(item) {
        Expression::Star(_) => true,
        Expression::Alias(alias) => matches!(alias.this, Expression::Star(_)),
        Expression::Column(column) => column.name.name == WILDCARD && !column.name.quoted,
        Expression::Dot(dot) => dot.field.name == WILDCARD && !dot.field.quoted,
        _ => false,
    }
}

/// Whether `star` is a wildcard alone, with no qualifier and no modifiers.
pub(crate) fn is_bare_star(item: &Expression) -> bool {
    matches!(unwrapped(item), Expression::Star(star) if star.table.is_none() && !is_modified(star))
}

/// The asterisk of wildcard `item`, written in `written`.
pub(crate) fn asterisk(item: &Expression, written: Span) -> Option<Span> {
    let item: &Expression = unwrapped(item);
    let star: &Star = match item {
        Expression::Star(star) => star,
        Expression::Alias(alias) => match &alias.this {
            Expression::Star(star) => star,
            _ => return None,
        },
        _ if is_wildcard(item) => return Some(last_character(written)),
        _ => return None,
    };
    let whole: Span = star.span.0.unwrap_or(written);
    let start: usize = match (&star.table, is_modified(star)) {
        (None, _) => whole.start,
        (Some(_), false) => return Some(last_character(whole)),
        (Some(_), true) => return Some(whole),
    };
    Some(Span {
        start,
        end: start + 1,
        ..whole
    })
}

fn last_character(span: Span) -> Span {
    Span {
        start: span.end.saturating_sub(1),
        ..span
    }
}

fn is_modified(star: &Star) -> bool {
    star.except.is_some()
        || star.replace.is_some()
        || star.rename.is_some()
        || star.ilike.is_some()
        || star.qualified.is_some()
}
