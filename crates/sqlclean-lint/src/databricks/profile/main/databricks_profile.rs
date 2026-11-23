//! `databricks_profile`: how lint reads Databricks SQL.

use crate::databricks::profile::_helpers::strict::strict_parse;
use crate::databricks::profile::_helpers::tokenizer::DATABRICKS_TOKENIZER;
use crate::databricks::profile::constants::{
    AGGREGATE_CALLS, FOREIGN_AGGREGATE_CALLS, SINGLE_ARGUMENT_AGGREGATES,
};
use crate::dialects::main::native_statements::native_statements;
use sqlquack_semantics::dialects::constants::DATABRICKS;
use sqlquack_semantics::dialects::types::DialectProfile;

use crate::dialects::models::LintProfile;

/// How Databricks SQL is read, and the lint facts only it has.
pub(crate) fn databricks_profile() -> LintProfile {
    LintProfile {
        tokens: |sql| {
            DATABRICKS_TOKENIZER
                .tokenize(sql)
                .map_err(|error| error.to_string())
        },
        statements: |sql, options, _| native_statements(sql, strict_parse, options),
        negated_distinct_from: true,
        foreign_aggregates: &FOREIGN_AGGREGATE_CALLS,
        plain_call_aggregates: true,
        named_aggregates: &AGGREGATE_CALLS,
        single_argument_aggregates: &SINGLE_ARGUMENT_AGGREGATES,
        json_parse_functions: &["PARSE_JSON", "TRY_PARSE_JSON"],
        resolved_name: |name, quoted| DATABRICKS.resolved_identifier(name, quoted),
        same_name: |left, right| DATABRICKS.same_identifier(left, right),
        identifier_quote: '`',
        union_distinct: true,
    }
}
