//! `long_literals`: SQLCLEAN044's literals over the limit, counted as written, which no tree records.

use crate::models::StringLiteral;

/// The string literals longer than `limit` characters.
pub fn long_literals(literals: &[StringLiteral], limit: usize) -> Vec<StringLiteral> {
    literals
        .iter()
        .filter(|literal| literal.length > limit)
        .copied()
        .collect()
}
