//! Positional context of one authored token among its neighbours.

use crate::sql_tokens::_helpers::reserved_keywords::is_reserved_keyword;
use crate::sql_tokens::constants::{
    ALIAS_KEYWORD, CALL_CLOSER, CALL_OPENER, COLUMN_LIST_RELATION_KEYWORDS, CTE_BODY_OPENERS,
    PLACEHOLDER_PREFIX,
};
use crate::sql_tokens::main::is_unquoted_word::is_unquoted_word;

pub(crate) struct TokenContext<'a> {
    pub(crate) raws: &'a [String],
    pub(crate) uppers: &'a [String],
    pub(crate) closers: &'a [Option<usize>],
    pub(crate) index: usize,
}

impl TokenContext<'_> {
    /// Whether the token is a reserved keyword or a called name, as SQLQuack's fingerprint folds.
    pub(crate) fn is_foldable(&self, dialect: &str) -> bool {
        let raw = self.raws[self.index].as_str();
        is_unquoted_word(raw)
            && (is_reserved_keyword(dialect, &self.uppers[self.index]) || self.calls(raw))
    }

    /// Whether the word is followed by `(` and is neither a placeholder nor a relation's column list.
    fn calls(&self, raw: &str) -> bool {
        let index = self.index;
        if self.uppers.get(index + 1).is_none_or(|next| next != CALL_OPENER)
            || raw.starts_with(PLACEHOLDER_PREFIX)
        {
            return false;
        }
        let previous: Option<&str> = index.checked_sub(1).map(|before| self.uppers[before].as_str());
        if previous.is_some_and(|word| COLUMN_LIST_RELATION_KEYWORDS.contains(&word)) {
            return false;
        }
        let Some(closer) = self.closers[index + 1] else {
            return true;
        };
        !(self.uppers.get(closer + 1).is_some_and(|next| next == ALIAS_KEYWORD)
            && self
                .uppers
                .get(closer + 2)
                .is_some_and(|next| CTE_BODY_OPENERS.contains(&next.as_str())))
    }
}

/// The index of the `)` closing each `(`, by the index of the `(`.
pub(crate) fn matching_closers(uppers: &[String]) -> Vec<Option<usize>> {
    let mut closers: Vec<Option<usize>> = vec![None; uppers.len()];
    let mut open: Vec<usize> = Vec::new();
    for (index, token) in uppers.iter().enumerate() {
        if token == CALL_OPENER {
            open.push(index);
        } else if token == CALL_CLOSER
            && let Some(opener) = open.pop()
        {
            closers[opener] = Some(index);
        }
    }
    closers
}
