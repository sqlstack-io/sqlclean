//! Token positions that decide which authored words may change letter case.

use sqlquack_syntax::tokens::models::TokenType;

/// Column aliases after `AS` may be any keyword in DuckDB and PostgreSQL.
pub(crate) const ALIAS_KEYWORD: &str = "AS";
/// The token that turns a preceding name into a function call.
pub(crate) const CALL_OPENER: &str = "(";
/// The token that closes a call's arguments or a column list.
pub(crate) const CALL_CLOSER: &str = ")";
/// Keywords after which a word followed by `(` names a relation and its column list.
pub(crate) const COLUMN_LIST_RELATION_KEYWORDS: [&str; 2] = ["INTO", "TABLE"];
/// The tokens that can follow `AS` after a CTE's column list.
pub(crate) const CTE_BODY_OPENERS: [&str; 3] = ["(", "MATERIALIZED", "NOT"];
/// The prefix of placeholder calls such as SQLBuild's `__ref` and `__source`.
pub(crate) const PLACEHOLDER_PREFIX: &str = "__";
/// Field names after `.`, path keys after `:`, and parameter names after `@` or `$`.
pub(crate) const NAME_PREFIXES: [&str; 4] = [".", ":", "@", "$"];
pub(crate) const PROJECTION_ENDS: [TokenType; 13] = [
    TokenType::From,
    TokenType::Where,
    TokenType::Group,
    TokenType::GroupBy,
    TokenType::Having,
    TokenType::Qualify,
    TokenType::Order,
    TokenType::OrderBy,
    TokenType::Limit,
    TokenType::Window,
    TokenType::Union,
    TokenType::Except,
    TokenType::Intersect,
];
/// Dialect names fingerprinted under another name, as SQLBuild resolves them.
pub(crate) const FINGERPRINT_DIALECT_ALIASES: [(&str, &str); 2] =
    [("motherduck", "duckdb"), ("sqlserver", "tsql")];
