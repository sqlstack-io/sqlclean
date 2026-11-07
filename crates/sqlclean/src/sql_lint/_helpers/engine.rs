//! Token nesting depths, the one lint helper the formatter still reads.

use polyglot_sql::tokens::{Token, TokenType};

/// Each token's bracket nesting depth, counted before the token itself.
pub(crate) fn token_depths(tokens: &[Token]) -> Vec<usize> {
    let mut depth = 0_usize;
    tokens
        .iter()
        .map(|token| {
            let current = depth;
            if matches!(
                token.token_type,
                TokenType::LParen | TokenType::LBracket | TokenType::LBrace
            ) {
                depth += 1;
            } else if matches!(
                token.token_type,
                TokenType::RParen | TokenType::RBracket | TokenType::RBrace
            ) {
                depth = depth.saturating_sub(1);
            }
            current
        })
        .collect()
}
