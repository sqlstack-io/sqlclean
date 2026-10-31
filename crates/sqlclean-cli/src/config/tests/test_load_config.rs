use std::fs;

use crate::config::main::load_config::load_config;
use crate::config::tests::test_types::{LoadedConfigTestCase, RejectedConfigTestCase};

#[test]
fn given_a_config_file_when_loading_from_below_then_the_nearest_one_applies() -> Result<(), String>
{
    let test_cases = [
        LoadedConfigTestCase {
            description: "values set in a parent directory apply and unset thresholds keep defaults",
            config_text: "dialect = \"duckdb\"\n[format]\nline_width = 80\n[rules]\nselect = [\"SQLCLEAN004\"]\n[rules.thresholds]\nmax_literal_length = 40\n",
            start: "models/orders",
            expected_dialect: "duckdb",
            expected_line_width: 80,
            expected_select: Some(&["SQLCLEAN004"]),
            expected_max_literal_length: 40,
            expected_max_ranking_order_by: 5,
        },
        LoadedConfigTestCase {
            description: "an empty file gives the defaults",
            config_text: "",
            start: "",
            expected_dialect: "generic",
            expected_line_width: 100,
            expected_select: None,
            expected_max_literal_length: 100,
            expected_max_ranking_order_by: 5,
        },
    ];
    for test_case in &test_cases {
        let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
        let start = directory.path().join(test_case.start);
        fs::create_dir_all(&start).map_err(|error| error.to_string())?;
        fs::write(
            directory.path().join("sqlclean.toml"),
            test_case.config_text,
        )
        .map_err(|error| error.to_string())?;
        let config = load_config(None, &start)?;
        let select: Option<Vec<&str>> = config
            .rules
            .select
            .as_ref()
            .map(|codes| codes.iter().map(String::as_str).collect());
        assert_eq!(config.root, directory.path(), "{}", test_case.description);
        assert_eq!(
            config.dialect, test_case.expected_dialect,
            "{}",
            test_case.description
        );
        assert_eq!(
            config.line_width, test_case.expected_line_width,
            "{}",
            test_case.description
        );
        assert_eq!(
            select,
            test_case.expected_select.map(<[&str]>::to_vec),
            "{}",
            test_case.description
        );
        assert_eq!(
            config.rules.thresholds.max_literal_length, test_case.expected_max_literal_length,
            "{}",
            test_case.description
        );
        assert_eq!(
            config.rules.thresholds.max_ranking_order_by, test_case.expected_max_ranking_order_by,
            "{}",
            test_case.description
        );
    }
    Ok(())
}

#[test]
fn given_invalid_values_when_loading_then_each_is_rejected_with_its_reason() -> Result<(), String> {
    let test_cases = [
        RejectedConfigTestCase {
            description: "an unknown dialect",
            config_text: "dialect = \"nosuch\"\n",
            expected_error: "unknown SQL dialect 'nosuch'",
        },
        RejectedConfigTestCase {
            description: "an unknown rules key",
            config_text: "[rules]\nunknown = 1\n",
            expected_error: "unknown field `unknown`",
        },
        RejectedConfigTestCase {
            description: "an exception with a blank reason",
            config_text: "[[rules.rule_exceptions]]\nrule = \"SQLCLEAN004\"\npath = \"a.sql\"\nreason = \" \"\n",
            expected_error: "needs a reason",
        },
        RejectedConfigTestCase {
            description: "a zero line width",
            config_text: "[format]\nline_width = 0\n",
            expected_error: "line_width must be positive",
        },
        RejectedConfigTestCase {
            description: "an indent of zero is rejected",
            config_text: "[format]\nindent = 0\n",
            expected_error: "indent must be between 1 and 8",
        },
    ];
    for test_case in &test_cases {
        let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
        fs::write(
            directory.path().join("sqlclean.toml"),
            test_case.config_text,
        )
        .map_err(|error| error.to_string())?;
        let error: String = load_config(None, directory.path())
            .err()
            .unwrap_or_default();
        assert!(
            error.contains(test_case.expected_error),
            "{}: {error}",
            test_case.description
        );
    }
    Ok(())
}
