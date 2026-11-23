//! The profile of a dialect the crate was built without.

use crate::dialects::models::LintProfile;
use crate::lint::constants::UNBUILT_DIALECT_FAILURE;

/// A profile that reads nothing, for a dialect built without its feature.
pub(crate) fn unbuilt_profile() -> LintProfile {
    LintProfile {
        tokens: |_| Err(UNBUILT_DIALECT_FAILURE.to_owned()),
        statements: |_, _, _| Err(UNBUILT_DIALECT_FAILURE.to_owned()),
        negated_distinct_from: false,
        foreign_aggregates: &[],
        plain_call_aggregates: false,
        named_aggregates: &[],
        single_argument_aggregates: &[],
        json_parse_functions: &[],
        resolved_name: |name, _| name.to_owned(),
        same_name: |left, right| left == right,
        identifier_quote: '"',
        union_distinct: false,
    }
}
