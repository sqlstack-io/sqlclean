use serde_json::{Value, json};

use sqlquack_parse::parsing::constants::CALL_DEPTH_CEILING_NOTE;

use crate::sql_lint::main::engine::lint_json;
use crate::sql_lint::main::formatter::format_json;
use crate::sql_lint::tests::{helpers, test_types};

#[test]
fn given_nested_function_calls_when_linting_then_the_requested_or_default_depth_bounds_them() {
    let test_cases = [
        test_types::FunctionDepthTestCase {
            description: "the default depth of 256 parses",
            depth: 256,
            max_function_call_depth: None,
            expected_error: None,
        },
        test_types::FunctionDepthTestCase {
            description: "one call past the default fails and names the setting",
            depth: 257,
            max_function_call_depth: None,
            expected_error: Some("configured limit 256; raise max_function_call_depth, up to 512"),
        },
        test_types::FunctionDepthTestCase {
            description: "a requested depth bounds the parse",
            depth: 65,
            max_function_call_depth: Some(64),
            expected_error: Some("configured limit 64; raise max_function_call_depth"),
        },
        test_types::FunctionDepthTestCase {
            description: "a requested depth above the default lets deeper calls parse",
            depth: 300,
            max_function_call_depth: Some(300),
            expected_error: None,
        },
    ];
    for test_case in test_cases {
        let request = json!({
            "version": 1,
            "sql": helpers::nested_function_sql(test_case.depth),
            "dialect": "snowflake",
            "max_function_call_depth": test_case.max_function_call_depth,
        });
        let outcome: Result<String, String> = lint_json(&request.to_string());
        let error: String = outcome.clone().err().unwrap_or_default();
        let diagnostics: Value = outcome
            .ok()
            .and_then(|response| serde_json::from_str::<Value>(&response).ok())
            .map_or_else(|| json!([]), |response| response["diagnostics"].clone());
        assert_eq!(
            !error.is_empty(),
            test_case.expected_error.is_some(),
            "{}: {error}",
            test_case.description
        );
        assert!(
            error.contains(test_case.expected_error.unwrap_or_default()),
            "{}: {error}",
            test_case.description
        );
        assert_eq!(diagnostics, json!([]), "{}", test_case.description);
    }
}

#[test]
fn given_sql_past_the_512_ceiling_when_formatting_then_only_sql_with_calls_explains_the_ceiling() {
    // The shared parser refuses with its guard code; the native dialects' formatter declines in its own words.
    let test_cases = [
        test_types::FormatCeilingTestCase {
            description: "snowflake: 512 nested calls are declined, explaining the ceiling once",
            dialect: "snowflake",
            sql: helpers::nested_function_sql(512),
            expected_reason: "value 513 exceeds configured limit 512",
            expected_notes: 1,
        },
        test_types::FormatCeilingTestCase {
            description: "snowflake: 600 UNION ALL operands are declined without the ceiling note",
            dialect: "snowflake",
            sql: helpers::union_all_sql(600),
            expected_reason: "value 513 exceeds configured limit 512",
            expected_notes: 0,
        },
        test_types::FormatCeilingTestCase {
            description: "duckdb: 512 nested calls are declined, explaining the ceiling once",
            dialect: "duckdb",
            sql: helpers::nested_function_sql(512),
            expected_reason: "value 513 exceeds configured limit 512",
            expected_notes: 1,
        },
        test_types::FormatCeilingTestCase {
            description: "duckdb: 600 UNION ALL operands are declined without the ceiling note",
            dialect: "duckdb",
            sql: helpers::union_all_sql(600),
            expected_reason: "value 513 exceeds configured limit 512",
            expected_notes: 0,
        },
        test_types::FormatCeilingTestCase {
            description: "postgres: 512 nested calls are declined by the native formatter",
            dialect: "postgres",
            sql: helpers::nested_function_sql(512),
            expected_reason: "512 levels deep",
            expected_notes: 0,
        },
        test_types::FormatCeilingTestCase {
            description: "postgres: 600 UNION ALL operands are declined without the ceiling note",
            dialect: "postgres",
            sql: helpers::union_all_sql(600),
            expected_reason: "512 levels deep",
            expected_notes: 0,
        },
        test_types::FormatCeilingTestCase {
            description: "bigquery: 512 nested calls are declined by the native formatter",
            dialect: "bigquery",
            sql: helpers::nested_function_sql(512),
            expected_reason: "512 levels deep",
            expected_notes: 0,
        },
        test_types::FormatCeilingTestCase {
            description: "bigquery: 600 UNION ALL operands are declined without the ceiling note",
            dialect: "bigquery",
            sql: helpers::union_all_sql(600),
            expected_reason: "512 levels deep",
            expected_notes: 0,
        },
        test_types::FormatCeilingTestCase {
            description: "databricks: 512 nested calls are declined by the native formatter",
            dialect: "databricks",
            sql: helpers::nested_function_sql(512),
            expected_reason: "512 levels deep",
            expected_notes: 0,
        },
        test_types::FormatCeilingTestCase {
            description: "databricks: 600 UNION ALL operands are declined without the ceiling note",
            dialect: "databricks",
            sql: helpers::union_all_sql(600),
            expected_reason: "512 levels deep",
            expected_notes: 0,
        },
    ];
    for test_case in &test_cases {
        let request = json!({
            "version": 1,
            "sql": test_case.sql,
            "dialect": test_case.dialect,
            "max_function_call_depth": 512,
        });
        let response: Value =
            serde_json::from_str(&format_json(&request.to_string()).expect("a format response"))
                .expect("JSON");
        let reason: &str = response["reason"].as_str().unwrap_or_default();
        assert_eq!(
            response["formatted"],
            json!(false),
            "{}: {reason}",
            test_case.description
        );
        assert!(
            reason.contains(test_case.expected_reason),
            "{}: {reason}",
            test_case.description
        );
        assert_eq!(
            reason.matches(CALL_DEPTH_CEILING_NOTE).count(),
            test_case.expected_notes,
            "{}: {reason}",
            test_case.description
        );
    }
}

#[test]
fn given_nested_function_calls_when_formatting_then_the_requested_or_default_depth_bounds_them() {
    let test_cases = [
        test_types::FunctionDepthTestCase {
            description: "the default depth of 256 formats",
            depth: 256,
            max_function_call_depth: None,
            expected_error: None,
        },
        test_types::FunctionDepthTestCase {
            description: "one call past the default is declined, naming the setting that raises it",
            depth: 257,
            max_function_call_depth: None,
            expected_error: Some("configured limit 256; raise max_function_call_depth, up to 512"),
        },
        test_types::FunctionDepthTestCase {
            description: "a requested depth bounds the formatter",
            depth: 65,
            max_function_call_depth: Some(64),
            expected_error: Some("configured limit 64"),
        },
        test_types::FunctionDepthTestCase {
            description: "a requested depth above the most allowed is declined",
            depth: 2,
            max_function_call_depth: Some(513),
            expected_error: Some("max_function_call_depth"),
        },
    ];
    for test_case in test_cases {
        let request = json!({
            "version": 1,
            "sql": helpers::nested_function_sql(test_case.depth),
            "dialect": "snowflake",
            "max_function_call_depth": test_case.max_function_call_depth,
        });
        let response: Value =
            serde_json::from_str(&format_json(&request.to_string()).expect("a format response"))
                .expect("JSON");
        let reason: &str = response["reason"].as_str().unwrap_or_default();
        assert_eq!(
            response["formatted"] == json!(false),
            test_case.expected_error.is_some(),
            "{}: {reason}",
            test_case.description
        );
        assert!(
            reason.contains(test_case.expected_error.unwrap_or_default()),
            "{}: {reason}",
            test_case.description
        );
    }
}
