//! The edits Rules return, rendered on the SQL only the engine holds.

use sqlclean_rules::models::RuleEdit;

use crate::lint::_helpers::engine::char_slice;
use crate::lint::models::LintEdit;

/// `edit` on `sql`; none when the layout it takes runs into a comment.
pub(super) fn rendered(edit: &RuleEdit, sql: &str) -> Option<LintEdit> {
    let end: usize = match edit.trailing_layout {
        true => layout_end(sql, edit.span.end)?,
        false => edit.span.end,
    };
    Some(LintEdit {
        start: edit.span.start,
        end,
        replacement: edit.replacement.to_owned(),
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
