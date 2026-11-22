//! Names written like columns that are none.

/// Values a name alone stands for.
pub(crate) const CONTEXT_VALUES: [&str; 8] = [
    "CURRENT_CATALOG",
    "CURRENT_DATE",
    "CURRENT_DATETIME",
    "CURRENT_ROLE",
    "CURRENT_SCHEMA",
    "CURRENT_TIME",
    "CURRENT_TIMESTAMP",
    "CURRENT_USER",
];
/// Calls whose first argument may be a date part written as a bare word.
pub(crate) const DATE_PART_CALLS: [&str; 10] = [
    "DATEADD",
    "DATEDIFF",
    "DATE_PART",
    "DATEPART",
    "TIMEADD",
    "TIMEDIFF",
    "TIMESTAMPADD",
    "TIMESTAMPDIFF",
    "TRY_DATEADD",
    "TRY_DATEDIFF",
];
/// The name of a star read as a column.
pub(crate) const WILDCARD: &str = "*";
