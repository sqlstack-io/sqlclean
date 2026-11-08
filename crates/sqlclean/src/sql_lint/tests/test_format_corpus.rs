use serde_json::Value;

use crate::sql_lint::tests::helpers::{check_dialect_corpus, check_unavailable_corpus};
use crate::sql_lint::tests::test_types;

const FORMAT_CORPUS: &str = include_str!("fixtures/format_corpus.json");

#[test]
fn given_dialect_corpus_when_formatting_then_output_layout_and_refusals_are_stable()
-> Result<(), String> {
    let test_cases = [
        test_types::FormatCorpusTestCase {
            description: "DuckDB identity corpus",
            dialect: "duckdb",
            expected_formatted: 189,
            expected_refused: 0,
        },
        test_types::FormatCorpusTestCase {
            description: "Snowflake identity corpus",
            dialect: "snowflake",
            expected_formatted: 427,
            expected_refused: 8,
        },
    ];
    let corpus: Vec<Value> =
        serde_json::from_str(FORMAT_CORPUS).map_err(|error| error.to_string())?;
    for test_case in test_cases {
        let (formatted, refused) = check_dialect_corpus(&corpus, test_case.dialect)?;
        assert_eq!(
            (formatted, refused),
            (test_case.expected_formatted, test_case.expected_refused),
            "{}",
            test_case.description
        );
    }
    Ok(())
}

/// Dialects SQLQuack does not format yet are reported unavailable, with every query left as written.
#[test]
fn given_dialects_without_a_native_formatter_when_formatting_then_each_query_is_unavailable()
-> Result<(), String> {
    let test_cases = [
        test_types::UnavailableCorpusTestCase {
            description: "BigQuery corpus",
            dialect: "bigquery",
            expected_unavailable: 168,
        },
        test_types::UnavailableCorpusTestCase {
            description: "Databricks corpus",
            dialect: "databricks",
            expected_unavailable: 62,
        },
        test_types::UnavailableCorpusTestCase {
            description: "PostgreSQL corpus",
            dialect: "postgres",
            expected_unavailable: 137,
        },
        test_types::UnavailableCorpusTestCase {
            description: "T-SQL corpus",
            dialect: "tsql",
            expected_unavailable: 75,
        },
    ];
    let corpus: Vec<Value> =
        serde_json::from_str(FORMAT_CORPUS).map_err(|error| error.to_string())?;
    for test_case in &test_cases {
        assert_eq!(
            check_unavailable_corpus(&corpus, test_case.dialect)?,
            test_case.expected_unavailable,
            "{}",
            test_case.description
        );
    }
    Ok(())
}
