//! CTE macro placeholders, which SQLQuack does not parse, taken out before layout and put back after.

use std::collections::BTreeSet;

use sqlquack_syntax::tokens::models::TokenType;
use sqlquack_syntax::{Dialect, LosslessToken, TokenKind};

use crate::constants::INTERPOLATION_PLACEHOLDER_PREFIX;

/// The separator token taken out with a placeholder.
const COMMA: &str = ",";
const RESTORE_FAILURE: &str = "native formatter could not restore CTE macro placeholders";

/// SQL with its CTE macro placeholders taken out, and the groups to put back.
pub(super) struct WithoutCteMacros {
    pub sql: String,
    groups: Vec<MacroGroup>,
    /// The significant and comment texts of the original SQL, in order.
    original: Vec<(bool, String)>,
}

/// Placeholders, with the commas taken out with them, that stood before one significant token.
struct MacroGroup {
    /// The index, among the significant tokens of the stripped SQL, of the token they preceded.
    anchor: usize,
    /// How many comments stood between the previous significant token and the group.
    comments_before: usize,
    texts: Vec<String>,
}

/// Take the CTE macro placeholders out of `sql`; `None` when it has none.
pub(super) fn take_out_cte_macros(
    sql: &str,
    dialect: Dialect,
) -> Result<Option<WithoutCteMacros>, String> {
    if !sql.contains(INTERPOLATION_PLACEHOLDER_PREFIX) {
        return Ok(None);
    }
    let Ok(tokens) = sqlquack_syntax::tokenize(sql, dialect) else {
        return Ok(None);
    };
    let significant: Vec<usize> = (0..tokens.len())
        .filter(|&index| matches!(tokens[index].kind, TokenKind::Significant(_)))
        .collect();
    let macros = macro_positions(sql, &tokens, &significant);
    if macros.is_empty() {
        return Ok(None);
    }
    let removed = removed_positions(&tokens, &significant, &macros);
    let mut groups: Vec<MacroGroup> = Vec::new();
    let mut stripped = String::with_capacity(sql.len());
    let mut kept = 0_usize;
    let mut cursor = 0_usize;
    let mut position = 0_usize;
    while position < significant.len() {
        if !removed.contains(&position) {
            kept += 1;
            position += 1;
            continue;
        }
        let previous_end = position
            .checked_sub(1)
            .map_or(0, |previous| tokens[significant[previous]].end);
        let comments_before = tokens[..significant[position]]
            .iter()
            .filter(|token| token.start >= previous_end && is_comment(token))
            .count();
        let mut texts: Vec<String> = Vec::new();
        while removed.contains(&position) {
            let index = significant[position];
            stripped.push_str(&sql[cursor..tokens[index].start]);
            texts.push(tokens[index].text(sql).to_string());
            cursor = tokens
                .get(index + 1)
                .filter(|next| next.kind == TokenKind::Whitespace)
                .map_or(tokens[index].end, |next| next.end);
            position += 1;
        }
        groups.push(MacroGroup {
            anchor: kept,
            comments_before,
            texts,
        });
    }
    stripped.push_str(&sql[cursor..]);
    Ok(Some(WithoutCteMacros {
        sql: stripped,
        groups,
        original: token_texts(sql, &tokens),
    }))
}

impl WithoutCteMacros {
    /// Put the placeholders back into `formatted`, the laid-out stripped SQL, each on its own line.
    pub(super) fn put_back(&self, formatted: &str, dialect: Dialect) -> Result<String, String> {
        let tokens = sqlquack_syntax::tokenize(formatted, dialect)
            .map_err(|_| RESTORE_FAILURE.to_string())?;
        let significant: Vec<usize> = (0..tokens.len())
            .filter(|&index| matches!(tokens[index].kind, TokenKind::Significant(_)))
            .collect();
        let mut restored = String::with_capacity(formatted.len() + 64);
        let mut cursor = 0_usize;
        for group in &self.groups {
            let gap_start = group
                .anchor
                .checked_sub(1)
                .map_or(0, |previous| tokens[significant[previous]].end);
            let gap_end = significant
                .get(group.anchor)
                .map_or(formatted.len(), |&anchor| tokens[anchor].start);
            let insert_at = match group.comments_before.checked_sub(1) {
                None => gap_start,
                Some(last) => tokens
                    .iter()
                    .filter(|token| {
                        token.start >= gap_start && token.end <= gap_end && is_comment(token)
                    })
                    .nth(last)
                    .map_or(gap_start, |comment| comment.end),
            };
            let rest = &formatted[insert_at..gap_end];
            let lead_end = insert_at + (rest.len() - rest.trim_start().len());
            let indent = " ".repeat(line_indentation(formatted, gap_end));
            let lead = &formatted[insert_at..lead_end];
            let mut separator = if lead.contains('\n') {
                lead.to_string()
            } else {
                format!("\n{indent}")
            };
            restored.push_str(&formatted[cursor..insert_at]);
            for text in &group.texts {
                if text != COMMA {
                    restored.push_str(&separator);
                    separator = format!("\n\n{indent}");
                }
                restored.push_str(text);
            }
            if lead_end < formatted.len() {
                restored.push_str(&separator);
            }
            cursor = lead_end;
        }
        restored.push_str(&formatted[cursor..]);
        let restored_tokens = sqlquack_syntax::tokenize(&restored, dialect)
            .map_err(|_| RESTORE_FAILURE.to_string())?;
        let restored_texts = token_texts(&restored, &restored_tokens);
        let same_tokens = restored_texts.len() == self.original.len()
            && restored_texts.iter().zip(&self.original).all(
                |((comment, text), (original_comment, original))| {
                    comment == original_comment
                        && if *comment {
                            text == original
                        } else {
                            text.eq_ignore_ascii_case(original)
                        }
                },
            );
        if !same_tokens {
            return Err(RESTORE_FAILURE.to_string());
        }
        Ok(restored)
    }
}

/// Positions, among significant tokens, of placeholders that stand where a CTE would.
fn macro_positions(sql: &str, tokens: &[LosslessToken], significant: &[usize]) -> Vec<usize> {
    let kinds: Vec<TokenType> = significant
        .iter()
        .map(|&index| match tokens[index].kind {
            TokenKind::Significant(kind) => kind,
            _ => TokenType::Var,
        })
        .collect();
    let text = |position: usize| tokens[significant[position]].text(sql);
    let is_macro = |position: usize| text(position).starts_with(INTERPOLATION_PLACEHOLDER_PREFIX);
    let starts_cte = |position: usize| {
        position < kinds.len()
            && (is_macro(position)
                || (text(position)
                    .chars()
                    .all(|character| character.is_alphanumeric() || character == '_')
                    && matches!(
                        kinds.get(position + 1),
                        Some(TokenType::As | TokenType::LParen)
                    )))
    };
    let mut depth = 0_usize;
    let mut scopes: Vec<usize> = Vec::new();
    let mut positions: Vec<usize> = Vec::new();
    for (position, kind) in kinds.iter().enumerate() {
        match kind {
            TokenType::LParen | TokenType::LBracket | TokenType::LBrace => depth += 1,
            TokenType::RParen | TokenType::RBracket | TokenType::RBrace => {
                depth = depth.saturating_sub(1);
            }
            TokenType::With => scopes.push(depth),
            TokenType::Select => scopes.retain(|scope| *scope != depth),
            _ if is_macro(position)
                && scopes.contains(&depth)
                && position > 0
                && (matches!(
                    kinds[position - 1],
                    TokenType::With | TokenType::Comma | TokenType::Recursive
                ) || positions.last() == Some(&(position - 1)))
                && (matches!(
                    kinds.get(position + 1),
                    Some(TokenType::Comma | TokenType::Select)
                ) || starts_cte(position + 1)) =>
            {
                positions.push(position);
            }
            _ => {}
        }
    }
    positions
}

/// The placeholders with each one's following comma, or the comma before a group ending the CTE list.
fn removed_positions(
    tokens: &[LosslessToken],
    significant: &[usize],
    macros: &[usize],
) -> BTreeSet<usize> {
    let is_comma = |position: usize| {
        significant
            .get(position)
            .is_some_and(|&index| tokens[index].kind == TokenKind::Significant(TokenType::Comma))
    };
    let mut removed: BTreeSet<usize> = macros.iter().copied().collect();
    for &position in macros {
        if is_comma(position + 1) {
            removed.insert(position + 1);
        }
    }
    for &position in macros {
        let group_end = !removed.contains(&(position + 1));
        if group_end && !is_comma(position + 1) {
            let mut first = position;
            while first > 0 && removed.contains(&(first - 1)) {
                first -= 1;
            }
            let ends_cte_list = significant.get(position + 1).is_none_or(|&index| {
                tokens[index].kind == TokenKind::Significant(TokenType::Select)
            });
            if first > 0 && is_comma(first - 1) && ends_cte_list {
                removed.insert(first - 1);
            }
        }
    }
    removed
}

fn is_comment(token: &LosslessToken) -> bool {
    matches!(token.kind, TokenKind::LineComment | TokenKind::BlockComment)
}

/// Each significant or comment token's text, with whether it is a comment.
fn token_texts(sql: &str, tokens: &[LosslessToken]) -> Vec<(bool, String)> {
    tokens
        .iter()
        .filter(|token| token.kind != TokenKind::Whitespace)
        .map(|token| (is_comment(token), token.text(sql).to_string()))
        .collect()
}

/// The spaces and tabs that start the line holding byte `offset`.
fn line_indentation(sql: &str, offset: usize) -> usize {
    let line_start = sql[..offset].rfind('\n').map_or(0, |position| position + 1);
    sql[line_start..offset]
        .chars()
        .take_while(|character| *character == ' ' || *character == '\t')
        .count()
}
