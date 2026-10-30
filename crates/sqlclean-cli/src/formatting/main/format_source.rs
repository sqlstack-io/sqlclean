//! Format one SQL file: each statement in canonical layout, separated by one blank line.

use sqlclean::sql_facts::main::file_facts::file_facts;
use sqlclean::sql_lint::constants::LINT_API_VERSION;
use sqlclean::sql_lint::main::format_requests::format_requests;
use sqlclean::sql_lint::models::{FormatRequest, FormatResponse};

use crate::formatting::constants::{FORMAT_REFUSAL as REFUSAL, STATEMENT_SEPARATOR};

/// `text` in canonical layout ending in one newline, or why the formatter left it unchanged.
pub fn format_source(text: &str, dialect: &str, line_width: usize) -> Result<String, String> {
    let facts = file_facts(text, dialect)?;
    let characters: Vec<char> = text.chars().collect();
    let mut boundaries: Vec<usize> = facts
        .statements
        .iter()
        .map(|statement| statement.terminated_end)
        .collect();
    if let Some(last) = boundaries.last_mut() {
        *last = characters.len();
    }
    let mut chunks: Vec<String> = Vec::new();
    let mut start: usize = 0;
    for end in boundaries {
        let chunk: String = characters[start..end].iter().collect();
        start = end;
        let formatted: String = format_statement(chunk.trim(), dialect, line_width)?;
        if !formatted.trim().is_empty() {
            chunks.push(formatted.trim_end().to_owned());
        }
    }
    if chunks.is_empty() {
        return Ok(text.trim().to_owned());
    }
    Ok(chunks.join(STATEMENT_SEPARATOR) + "\n")
}

fn format_statement(sql: &str, dialect: &str, line_width: usize) -> Result<String, String> {
    let request = FormatRequest {
        version: LINT_API_VERSION,
        sql: sql.to_owned(),
        dialect: dialect.to_owned(),
        line_width: Some(line_width),
        ..FormatRequest::default()
    };
    let response: FormatResponse = format_requests(vec![request])
        .pop()
        .ok_or_else(|| REFUSAL.to_owned())?;
    if response.formatted {
        Ok(response.sql)
    } else {
        Err(response.reason.unwrap_or_else(|| REFUSAL.to_owned()))
    }
}
