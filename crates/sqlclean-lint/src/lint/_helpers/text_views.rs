//! The three narrow views of the text: terminators, literal lengths and comments, none of which can rebuild structure.

use sqlclean_rules::models::{CommentPlacement, StringLiteral, Terminator};
use sqlquack_syntax::tokens::models::{Token, TokenType};

use crate::lint::_helpers::rule_edits::char_slice;

/// Each terminator, and whether it only follows another.
pub(super) fn terminators(tokens: &[Token]) -> Vec<Terminator> {
    let mut found: Vec<Terminator> = Vec::new();
    let mut follows_terminator: bool = false;
    for token in tokens {
        match token.token_type {
            TokenType::Space
            | TokenType::Break
            | TokenType::LineComment
            | TokenType::BlockComment => {}
            TokenType::Semicolon => {
                found.push(Terminator {
                    span: token.span,
                    follows_terminator,
                });
                follows_terminator = true;
            }
            _ => follows_terminator = false,
        }
    }
    found
}

/// Each string literal's range and length, without its text.
pub(super) fn literals(tokens: &[Token]) -> Vec<StringLiteral> {
    tokens
        .iter()
        .filter(|token| is_string(token.token_type))
        .map(|token| StringLiteral {
            span: token.span,
            length: token.text.chars().count(),
        })
        .collect()
}

/// The comments beside each token, as placements.
pub(super) fn comments(tokens: &[Token], sql: &str) -> Vec<CommentPlacement> {
    let mut found: Vec<CommentPlacement> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if !token.trailing_comments.is_empty() {
            found.push(CommentPlacement {
                anchor: token.span,
                trailing: true,
                block: false,
                detached: false,
                before_punctuation: false,
            });
        }
        if token.comments.is_empty() {
            continue;
        }
        let before: usize = index
            .checked_sub(1)
            .map_or(0, |previous| tokens[previous].span.end);
        let gap: &str = char_slice(sql, before, token.span.start).unwrap_or_default();
        found.push(CommentPlacement {
            anchor: token.span,
            trailing: false,
            block: gap.contains("/*"),
            detached: gap.replace("\r\n", "\n").ends_with("\n\n"),
            before_punctuation: matches!(
                token.token_type,
                TokenType::Comma | TokenType::RParen | TokenType::Semicolon
            ),
        });
    }
    found
}

fn is_string(kind: TokenType) -> bool {
    matches!(
        kind,
        TokenType::String
            | TokenType::DollarString
            | TokenType::EscapeString
            | TokenType::RawString
            | TokenType::NationalString
            | TokenType::UnicodeString
            | TokenType::ByteString
            | TokenType::TripleSingleQuotedString
            | TokenType::TripleDoubleQuotedString
            | TokenType::HeredocString
            | TokenType::HeredocStringAlternative
    )
}
