//! `bigquery_profile`: how lint reads BigQuery SQL.

use crate::bigquery::profile::_helpers::strict::strict_parse;
use crate::bigquery::profile::_helpers::tokenizer::BIGQUERY_TOKENIZER;
use crate::bigquery::profile::constants::{
    AGGREGATE_CALLS, FOREIGN_AGGREGATE_CALLS, SINGLE_ARGUMENT_AGGREGATES,
};
use crate::dialects::main::native_statements::native_statements;
use sqlquack_semantics::dialects::constants::BIGQUERY;
use sqlquack_semantics::dialects::types::DialectProfile;

use crate::dialects::models::LintProfile;

/// How BigQuery SQL is read, and the lint facts only it has.
pub(crate) fn bigquery_profile() -> LintProfile {
    LintProfile {
        tokens: |sql| {
            BIGQUERY_TOKENIZER
                .tokenize(sql)
                .map_err(|error| error.to_string())
        },
        statements: |sql, options, _| native_statements(sql, strict_parse, options),
        negated_distinct_from: true,
        foreign_aggregates: &FOREIGN_AGGREGATE_CALLS,
        plain_call_aggregates: true,
        named_aggregates: &AGGREGATE_CALLS,
        single_argument_aggregates: &SINGLE_ARGUMENT_AGGREGATES,
        json_parse_functions: &["PARSE_JSON", "SAFE.PARSE_JSON"],
        resolved_name: |name, quoted| BIGQUERY.resolved_identifier(name, quoted),
        same_name: |left, right| BIGQUERY.same_identifier(left, right),
        identifier_quote: '`',
        union_distinct: true,
    }
}
