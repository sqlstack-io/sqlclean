use sqlquack_syntax::tokens::models::Dialect;
use sqlquack_tokenize::lossless::models::TokenKind;

/// Significant token texts of `sql` under one of SQLQuack's tokenizers.
pub(crate) fn significant_texts(sql: &str, tokenizer: Dialect) -> Result<Vec<String>, String> {
    Ok(sqlquack_parse::tokenize(sql, tokenizer)
        .map_err(|error| error.to_string())?
        .iter()
        .filter(|token| matches!(token.kind, TokenKind::Significant(_)))
        .map(|token| token.text(sql).to_owned())
        .collect())
}
