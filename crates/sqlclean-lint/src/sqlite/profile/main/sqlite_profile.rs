//! `sqlite_profile`: how lint reads SQLite SQL.

use sqlquack_syntax::Dialect;

use crate::dialects::main::shared_statements::shared_statements;
use crate::dialects::main::shared_tokens::shared_tokens;
use sqlquack_semantics::dialects::constants::SQLITE;
use sqlquack_semantics::dialects::types::DialectProfile;

use crate::dialects::models::LintProfile;

/// How SQLite SQL is read, and the lint facts only it has.
pub(crate) fn sqlite_profile() -> LintProfile {
    LintProfile {
        tokens: |sql| shared_tokens(sql, Dialect::SQLite),
        statements: |sql, options, mode| shared_statements(sql, Dialect::SQLite, (options, mode)),
        negated_distinct_from: false,
        foreign_aggregates: &[],
        plain_call_aggregates: false,
        named_aggregates: &[],
        single_argument_aggregates: &[],
        json_parse_functions: &[],
        resolved_name: |name, quoted| SQLITE.resolved_identifier(name, quoted),
        same_name: |left, right| SQLITE.same_identifier(left, right),
        identifier_quote: '"',
        union_distinct: false,
    }
}
