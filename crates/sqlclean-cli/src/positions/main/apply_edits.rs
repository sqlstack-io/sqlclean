//! Apply non-overlapping character-offset edits to a text.

use crate::positions::models::Edit;

/// `text` with every edit applied; `edits` must be sorted by start and must not overlap.
pub fn apply_edits(text: &str, edits: &[Edit]) -> String {
    let characters: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len());
    let mut cursor: usize = 0;
    for edit in edits {
        let start: usize = edit.start.min(characters.len()).max(cursor);
        result.extend(&characters[cursor..start]);
        result.push_str(&edit.text);
        cursor = edit.end.min(characters.len()).max(start);
    }
    result.extend(&characters[cursor..]);
    result
}
