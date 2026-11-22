//! `is_generated`: whether a name stands for template code.

use crate::models::RuleInput;

/// Whether `name` is a placeholder for a name the host generates, which Rules never judge as authored.
pub(crate) fn is_generated(input: &RuleInput<'_>, name: &str) -> bool {
    let lowered: String = name.to_ascii_lowercase();
    input
        .settings
        .placeholders
        .generated
        .iter()
        .any(|prefix| lowered.starts_with(prefix))
}
