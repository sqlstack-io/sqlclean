//! Statement source ranges from the token stream.

use polyglot_sql::tokens::{Token, TokenType};

/// The first and last token index of one statement, and the end of its terminator.
pub(crate) struct StatementTokens {
    pub(crate) first: usize,
    pub(crate) last: usize,
    pub(crate) terminated_end: usize,
}

/// The tokens of each non-empty statement, split at semicolons.
pub(crate) fn statement_token_ranges(tokens: &[Token]) -> Vec<StatementTokens> {
    let mut ranges: Vec<StatementTokens> = Vec::new();
    let mut first: Option<usize> = None;
    for (index, token) in tokens.iter().enumerate() {
        if token.token_type == TokenType::Semicolon {
            if let Some(start) = first.take() {
                ranges.push(StatementTokens {
                    first: start,
                    last: index - 1,
                    terminated_end: token.span.end,
                });
            }
        } else if first.is_none() {
            first = Some(index);
        }
    }
    if let Some(start) = first {
        let last: usize = tokens.len() - 1;
        ranges.push(StatementTokens {
            first: start,
            last,
            terminated_end: tokens[last].span.end,
        });
    }
    ranges
}
