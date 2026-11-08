//! Format one SQL file: each statement in canonical layout, separated by one blank line.

use sqlclean::sql_facts::main::statement_ends::statement_ends;
use sqlclean::sql_lint::constants::LINT_API_VERSION;
use sqlclean::sql_lint::main::format_requests::format_requests;
use sqlclean::sql_lint::models::{FormatRequest, FormatResponse};

use crate::formatting::constants::{FORMAT_REFUSAL as REFUSAL, STATEMENT_SEPARATOR};
use crate::formatting::models::Formatted;

/// `text` in canonical layout ending in one newline, or unavailable, or why the formatter refused it.
pub fn format_source(
    text: &str,
    dialect: &str,
    line_width: usize,
    indent: usize,
) -> Result<Formatted, String> {
    let characters: Vec<char> = text.chars().collect();
    let Some(mut boundaries) = statement_ends(text, dialect)? else {
        return Ok(Formatted::Unavailable(format!(
            "SQL formatting is unavailable for the {dialect} dialect: SQLQuack does not parse it yet"
        )));
    };
    if let Some(last) = boundaries.last_mut() {
        *last = characters.len();
    }
    let mut chunks: Vec<String> = Vec::new();
    let mut start: usize = 0;
    for end in boundaries {
        let chunk: String = characters[start..end].iter().collect();
        start = end;
        let formatted: String = match format_statement(chunk.trim(), dialect, line_width, indent)? {
            Formatted::Sql(sql) => sql,
            unavailable @ Formatted::Unavailable(_) => return Ok(unavailable),
        };
        if !formatted.trim().is_empty() {
            chunks.push(formatted.trim_end().to_owned());
        }
    }
    if chunks.is_empty() {
        let comments: &str = text.trim();
        return Ok(Formatted::Sql(if comments.is_empty() {
            String::new()
        } else {
            format!("{comments}\n")
        }));
    }
    Ok(Formatted::Sql(chunks.join(STATEMENT_SEPARATOR) + "\n"))
}

fn format_statement(
    sql: &str,
    dialect: &str,
    line_width: usize,
    indent: usize,
) -> Result<Formatted, String> {
    let request = FormatRequest {
        version: LINT_API_VERSION,
        sql: sql.to_owned(),
        dialect: dialect.to_owned(),
        line_width: Some(line_width),
        indent: Some(indent),
        ..FormatRequest::default()
    };
    let response: FormatResponse = format_requests(vec![request])
        .pop()
        .ok_or_else(|| REFUSAL.to_owned())?;
    if let Some(reason) = response.unavailable {
        Ok(Formatted::Unavailable(reason))
    } else if response.formatted {
        Ok(Formatted::Sql(response.sql))
    } else {
        Err(response.reason.unwrap_or_else(|| REFUSAL.to_owned()))
    }
}
