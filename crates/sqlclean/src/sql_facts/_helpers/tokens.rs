//! Function calls and literals, which the syntax tree does not locate, read from tokens.

use polyglot_sql::tokens::{Token, TokenType};

use crate::sql_facts::models::{LiteralNode, NamedNode, StarNode};

const STRING_KIND: &str = "string";
const NUMBER_KIND: &str = "number";
/// Words before `name (` that make the parentheses a column list rather than a call.
const COLUMN_LIST_OWNERS: [&str; 4] = ["INTO", "TABLE", "VIEW", "INDEX"];

/// Every call of a named function in `tokens`, located at its name.
pub(crate) fn function_nodes(tokens: &[Token]) -> Vec<NamedNode> {
    tokens
        .windows(2)
        .enumerate()
        .filter(|(index, pair)| {
            pair[1].token_type == TokenType::LParen
                && is_callable_name(&pair[0])
                && !owns_column_list(tokens, *index)
        })
        .map(|(_, pair)| NamedNode {
            name: pair[0].text.clone(),
            start: pair[0].span.start,
            end: pair[0].span.end,
        })
        .collect()
}

/// Every string and numeric literal in `tokens`.
pub(crate) fn literal_nodes(tokens: &[Token], sql: &[char]) -> Vec<LiteralNode> {
    tokens
        .iter()
        .filter_map(|token| {
            let kind: &str = match token.token_type {
                TokenType::String | TokenType::NationalString => STRING_KIND,
                TokenType::Number => NUMBER_KIND,
                _ => return None,
            };
            Some(LiteralNode {
                kind: kind.to_owned(),
                text: sql
                    .get(token.span.start..token.span.end)
                    .map_or_else(|| token.text.clone(), |chars| chars.iter().collect()),
                start: token.span.start,
                end: token.span.end,
            })
        })
        .collect()
}

/// Every `*` or `qualifier.*` projection in `tokens`; the parser leaves stars unlocated.
pub(crate) fn star_nodes(tokens: &[Token]) -> Vec<StarNode> {
    tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| token.token_type == TokenType::Star)
        .filter_map(|(index, star)| {
            let previous: &Token = tokens.get(index.checked_sub(1)?)?;
            if previous.token_type == TokenType::Dot {
                let qualifier: &Token = tokens.get(index.checked_sub(2)?)?;
                return Some(StarNode {
                    qualifier: Some(qualifier.text.clone()),
                    start: qualifier.span.start,
                    end: star.span.end,
                });
            }
            matches!(
                previous.token_type,
                TokenType::Select | TokenType::Comma | TokenType::Distinct | TokenType::All
            )
            .then(|| StarNode {
                qualifier: None,
                start: star.span.start,
                end: star.span.end,
            })
        })
        .collect()
}

fn is_callable_name(token: &Token) -> bool {
    let adjacent_word: bool = token
        .text
        .chars()
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_');
    adjacent_word
        && !matches!(
            token.token_type,
            TokenType::String | TokenType::Number | TokenType::NationalString
        )
        && !is_clause_keyword(token)
}

fn is_clause_keyword(token: &Token) -> bool {
    matches!(
        token.token_type,
        TokenType::In
            | TokenType::Exists
            | TokenType::As
            | TokenType::Values
            | TokenType::Over
            | TokenType::Using
            | TokenType::On
            | TokenType::From
            | TokenType::Join
            | TokenType::With
            | TokenType::Where
            | TokenType::And
            | TokenType::Or
            | TokenType::Not
            | TokenType::Select
            | TokenType::Union
            | TokenType::Window
            | TokenType::Lateral
    )
}

fn owns_column_list(tokens: &[Token], name_index: usize) -> bool {
    name_index
        .checked_sub(1)
        .and_then(|previous| tokens.get(previous))
        .is_some_and(|previous| {
            COLUMN_LIST_OWNERS
                .iter()
                .any(|owner| previous.text.eq_ignore_ascii_case(owner))
        })
}

/// The character span of the join clause whose relation starts at `relation_start`.
pub(crate) fn join_clause_span(tokens: &[Token], relation_start: usize) -> Option<(usize, usize)> {
    let relation: usize = tokens
        .iter()
        .position(|token| token.span.start >= relation_start)?;
    let start: usize = join_clause_start(tokens, relation);
    let end: usize = join_clause_end(tokens, start);
    Some((tokens[start].span.start, tokens[end].span.end))
}

/// The first token of the clause: its join keyword and modifiers, or the relation of a comma join.
fn join_clause_start(tokens: &[Token], relation: usize) -> usize {
    let keyword: Option<usize> = tokens[..relation]
        .iter()
        .rposition(|token| {
            matches!(
                token.token_type,
                TokenType::Join | TokenType::StraightJoin | TokenType::Comma | TokenType::From
            )
        })
        .filter(|index| {
            matches!(
                tokens[*index].token_type,
                TokenType::Join | TokenType::StraightJoin
            )
        });
    let Some(mut start) = keyword else {
        return relation;
    };
    while start > 0 && is_join_modifier(&tokens[start - 1]) {
        start -= 1;
    }
    start
}

/// The last token of the clause that begins at `start`, before the next clause at its depth.
fn join_clause_end(tokens: &[Token], start: usize) -> usize {
    let keywords: usize = tokens[start..]
        .iter()
        .take_while(|token| {
            is_join_modifier(token)
                || matches!(token.token_type, TokenType::Join | TokenType::StraightJoin)
        })
        .count();
    let mut depth: usize = 0;
    for index in start + keywords.max(1)..tokens.len() {
        let token: &Token = &tokens[index];
        match token.token_type {
            TokenType::LParen => depth += 1,
            TokenType::RParen if depth == 0 => return index - 1,
            TokenType::RParen => depth -= 1,
            _ if depth == 0 && ends_join_clause(tokens, index) => return index - 1,
            _ => {}
        }
    }
    tokens.len() - 1
}

fn is_join_modifier(token: &Token) -> bool {
    matches!(
        token.token_type,
        TokenType::Left
            | TokenType::Right
            | TokenType::Inner
            | TokenType::Outer
            | TokenType::Full
            | TokenType::Cross
            | TokenType::Natural
            | TokenType::Semi
            | TokenType::Anti
            | TokenType::Positional
    )
}

/// The token at `index` begins another join or clause, rather than continuing this one.
fn ends_join_clause(tokens: &[Token], index: usize) -> bool {
    let token: &Token = &tokens[index];
    let calls_function: bool = tokens
        .get(index + 1)
        .is_some_and(|next| next.token_type == TokenType::LParen);
    let starts_join: bool = (is_join_modifier(token) && !calls_function)
        || matches!(token.token_type, TokenType::Join | TokenType::StraightJoin);
    starts_join
        || matches!(
            token.token_type,
            TokenType::Comma
                | TokenType::Semicolon
                | TokenType::Where
                | TokenType::Group
                | TokenType::GroupBy
                | TokenType::Order
                | TokenType::OrderBy
                | TokenType::Having
                | TokenType::Limit
                | TokenType::Offset
                | TokenType::Fetch
                | TokenType::Qualify
                | TokenType::Window
                | TokenType::Union
                | TokenType::Except
                | TokenType::Intersect
                | TokenType::Pivot
                | TokenType::Unpivot
        )
}
