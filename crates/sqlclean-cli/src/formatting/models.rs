//! What formatting one file yields.

/// A file in canonical layout, or why its dialect cannot be formatted yet; a notice, not a refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Formatted {
    Sql(String),
    Unavailable(String),
}
