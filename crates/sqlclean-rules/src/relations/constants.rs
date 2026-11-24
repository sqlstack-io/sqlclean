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
/// Calls with an argument that may be a date part written as a bare word, and its position.
pub(crate) const DATE_PART_CALLS: [(&str, usize); 11] = [
    ("DATEADD", 0),
    ("DATEDIFF", 0),
    ("DATE_PART", 0),
    ("DATEPART", 0),
    ("LAST_DAY", 1),
    ("TIMEADD", 0),
    ("TIMEDIFF", 0),
    ("TIMESTAMPADD", 0),
    ("TIMESTAMPDIFF", 0),
    ("TRY_DATEADD", 0),
    ("TRY_DATEDIFF", 0),
];
/// The name of a star read as a column.
pub(crate) const WILDCARD: &str = "*";
/// The row an ON CONFLICT action reads.
pub(crate) const CONFLICT_ROW: &str = "EXCLUDED";
/// The rows a rule's actions read.
pub(crate) const RULE_ROWS: [&str; 2] = ["NEW", "OLD"];
