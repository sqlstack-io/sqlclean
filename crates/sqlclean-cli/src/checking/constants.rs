/// The finding for a file the parser rejects; it cannot be deselected or excepted.
pub const PARSE_ERROR_CODE: &str = "SQLCLEAN999";
pub(crate) const PARSE_ERROR_MESSAGE: &str = "SQL could not be parsed";
pub(crate) const PARSE_ERROR_REMEDIATION: &str =
    "Fix the syntax error, or set this file's dialect in sqlclean.toml.";
pub(crate) const CROSS_JOIN_CODE: &str = "SQLCLEAN038";
/// Custom Rule codes start with this prefix.
pub const CUSTOM_RULE_PREFIX: &str = "XSQLCLEAN";
/// Built-in Rules whose findings can carry a fix.
pub(crate) const FIXABLE_BUILTIN_RULES: [&str; 10] = [
    "SQLCLEAN001",
    "SQLCLEAN002",
    "SQLCLEAN003",
    "SQLCLEAN005",
    "SQLCLEAN006",
    "SQLCLEAN008",
    "SQLCLEAN013",
    "SQLCLEAN017",
    "SQLCLEAN025",
    "SQLCLEAN030",
];
/// Built-in Rules whose fix can change query results, so `--unsafe-fixes` must opt in.
pub(crate) const UNSAFE_FIX_BUILTIN_RULES: [&str; 1] = ["SQLCLEAN001"];
