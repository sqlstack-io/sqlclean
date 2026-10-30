/// The finding for a file the parser rejects; it cannot be deselected or excepted.
pub const PARSE_ERROR_CODE: &str = "SQC999";
pub(crate) const PARSE_ERROR_MESSAGE: &str = "SQL could not be parsed";
pub(crate) const PARSE_ERROR_REMEDIATION: &str =
    "Fix the syntax error, or set this file's dialect in sqlclean.toml.";
pub(crate) const CROSS_JOIN_CODE: &str = "SQC038";
/// Custom Rule codes start with this prefix.
pub const CUSTOM_RULE_PREFIX: &str = "XSQC";
/// Built-in Rules whose findings can carry a fix.
pub(crate) const FIXABLE_BUILTIN_RULES: [&str; 10] = [
    "SQC001", "SQC002", "SQC003", "SQC005", "SQC006", "SQC008", "SQC013", "SQC017", "SQC025",
    "SQC030",
];
/// Built-in Rules whose fix can change query results, so `--unsafe-fixes` must opt in.
pub(crate) const UNSAFE_FIX_BUILTIN_RULES: [&str; 1] = ["SQC001"];
