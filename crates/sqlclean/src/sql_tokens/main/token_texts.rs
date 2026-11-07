//! Authored text of tokens, taken from their character spans.

use std::ops::Range;

/// Return the authored text of every token span, in Unicode characters, of `sql`.
pub fn token_texts(
    sql: &str,
    spans: impl IntoIterator<Item = Range<usize>>,
) -> Result<Vec<String>, String> {
    let characters: Vec<char> = sql.chars().collect();
    spans
        .into_iter()
        .map(|span| {
            characters
                .get(span)
                .map(|slice| slice.iter().collect())
                .ok_or_else(|| "SQL tokenizer returned an invalid token span".to_string())
        })
        .collect()
}
