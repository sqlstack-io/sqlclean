//! Canonical token stream of authored token texts.

use crate::sql_tokens::_helpers::case_folding::foldable_tokens;
use crate::sql_tokens::models::CanonicalToken;

/// Upper-case the case-insensitive tokens among `raws`, the texts of the significant tokens.
pub fn canonical_tokens(raws: Vec<String>, dialect: &str) -> Vec<CanonicalToken> {
    let foldable = foldable_tokens(&raws, dialect);
    raws.into_iter()
        .zip(foldable)
        .map(|(raw, fold)| CanonicalToken {
            text: if fold { raw.to_ascii_uppercase() } else { raw },
        })
        .collect()
}
