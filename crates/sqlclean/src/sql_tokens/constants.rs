//! Token positions that decide which authored words may change letter case.

use polyglot_sql::tokens::TokenType;

use crate::sql_tokens::models::{FingerprintTokenizer, KeywordList, SqlquackFingerprintDialect};

/// Column aliases after `AS` may be any keyword in DuckDB and PostgreSQL.
pub(crate) const ALIAS_KEYWORD: &str = "AS";
/// The token that turns a preceding name into a function call.
pub(crate) const CALL_OPENER: &str = "(";
/// Field names after `.`, path keys after `:`, and parameter names after `@` or `$`.
pub(crate) const NAME_PREFIXES: [&str; 4] = [".", ":", "@", "$"];
/// Built-in calls with keyword syntax inside their parentheses, which catalogues do not list.
pub const CALL_SYNTAX_FUNCTIONS: [&str; 6] = [
    "CAST",
    "TRY_CAST",
    "SAFE_CAST",
    "EXTRACT",
    "CONVERT",
    "TRY_CONVERT",
];

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
/// Fingerprint dialects SQLQuack tokenizes; generic uses the DuckDB tokenizer, PostgreSQL its native scanner.
pub(crate) const SQLQUACK_FINGERPRINT_DIALECTS: [SqlquackFingerprintDialect; 6] = [
    SqlquackFingerprintDialect {
        name: "generic",
        tokenizer: FingerprintTokenizer::Lossless(sqlquack_syntax::Dialect::DuckDB),
        keywords: KeywordList::Generic,
    },
    SqlquackFingerprintDialect {
        name: "duckdb",
        tokenizer: FingerprintTokenizer::Lossless(sqlquack_syntax::Dialect::DuckDB),
        keywords: KeywordList::DuckDB,
    },
    SqlquackFingerprintDialect {
        name: "postgres",
        tokenizer: FingerprintTokenizer::PostgresScanner,
        keywords: KeywordList::PostgreSQL,
    },
    SqlquackFingerprintDialect {
        name: "postgresql",
        tokenizer: FingerprintTokenizer::PostgresScanner,
        keywords: KeywordList::PostgreSQL,
    },
    SqlquackFingerprintDialect {
        name: "snowflake",
        tokenizer: FingerprintTokenizer::Lossless(sqlquack_syntax::Dialect::Snowflake),
        keywords: KeywordList::Snowflake,
    },
    SqlquackFingerprintDialect {
        name: "sqlite",
        tokenizer: FingerprintTokenizer::Lossless(sqlquack_syntax::Dialect::SQLite),
        keywords: KeywordList::Generic,
    },
];
/// The token that closes a call's arguments or a column list.
pub(crate) const CALL_CLOSER: &str = ")";
/// Keywords after which a word followed by `(` names a relation and its column list.
pub(crate) const COLUMN_LIST_RELATION_KEYWORDS: [&str; 2] = ["INTO", "TABLE"];
/// The tokens that can follow `AS` after a CTE's column list.
pub(crate) const CTE_BODY_OPENERS: [&str; 3] = ["(", "MATERIALIZED", "NOT"];
/// The prefix of SQLBuild placeholder calls such as `__ref` and `__source`.
pub(crate) const PLACEHOLDER_PREFIX: &str = "__";
/// The keyword between a CTE's column list and its body.
pub(crate) const CTE_BODY_PREFIX: &str = "AS";
