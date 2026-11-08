//! The built-in Rules a family selector or an absent selection enables.

/// The codes enabled when no Rule selection is given.
pub fn default_rules() -> Vec<&'static str> {
    sqlquack_lint::default_rules()
}
