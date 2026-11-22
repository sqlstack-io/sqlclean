//! The edits Rules return, rendered on the SQL only the engine holds.

use sqlclean_rules::models::{Replacement, RuleEdit};
use sqlquack_syntax::tokens::models::{Span, Token};

use crate::lint::_helpers::engine::{char_slice, contains_comment};
use crate::lint::models::LintEdit;

/// `edit` on `sql`; none when it would lose or move a comment.
pub(super) fn rendered(edit: &RuleEdit, sql: &str, tokens: &[Token]) -> Option<LintEdit> {
    if holds_comment(edit.span, sql, tokens) {
        return None;
    }
    let end: usize = match edit.trailing_layout {
        true => layout_end(sql, edit.span.end)?,
        false => edit.span.end,
    };
    let replacement: String = match edit.replacement {
        Replacement::Text(text) => text.to_owned(),
        Replacement::Wrapped {
            before,
            kept,
            after,
        } => {
            let kept: &str = char_slice(sql, kept.start, kept.end)?.trim();
            format!("{before}{kept}{after}")
        }
    };
    Some(LintEdit {
        start: edit.span.start,
        end,
        replacement,
    })
}

/// Whether a comment in any of the dialect's spellings is inside `span`, which whole tokens bound.
fn holds_comment(span: Span, sql: &str, tokens: &[Token]) -> bool {
    if char_slice(sql, span.start, span.end).is_none_or(contains_comment) {
        return true;
    }
    tokens
        .iter()
        .filter(|token| token.span.start >= span.start && token.span.end <= span.end)
        .any(|token| {
            (token.span.start > span.start && !token.comments.is_empty())
                || (token.span.end < span.end && !token.trailing_comments.is_empty())
        })
}

/// Where the layout from character `start` ends; none when a comment begins there.
fn layout_end(sql: &str, start: usize) -> Option<usize> {
    let rest: &str = char_slice(sql, start, sql.chars().count())?;
    let end: usize = start + rest.chars().take_while(|next| next.is_whitespace()).count();
    let next: &str = rest.trim_start();
    let commented: bool = next.starts_with("--") || next.starts_with("/*");
    (!next.is_empty() && !commented).then_some(end)
}
