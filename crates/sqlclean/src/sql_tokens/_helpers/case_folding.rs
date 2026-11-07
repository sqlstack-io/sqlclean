//! Which authored tokens are case-insensitive keywords or called function names.

use crate::sql_tokens::_helpers::name_positions::name_positions;
use crate::sql_tokens::_helpers::token_context::{TokenContext, matching_closers};

/// For every token text, whether the formatter may recase it and the fingerprint folds it.
pub(crate) fn foldable_tokens(raws: &[String], dialect: &str) -> Vec<bool> {
    let uppers: Vec<String> = raws.iter().map(|raw| raw.to_ascii_uppercase()).collect();
    let closers = matching_closers(&uppers);
    name_positions(&uppers)
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            let context = TokenContext {
                raws,
                uppers: &uppers,
                closers: &closers,
                index,
            };
            !name && context.is_foldable(dialect)
        })
        .collect()
}
