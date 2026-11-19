use crate::sql_lint::tests::{helpers, test_types};

/// Databricks keeps a WINDOW clause written after a parenthesized query on the subquery, so a ranking call naming that window is still checked.
#[test]
fn given_databricks_window_after_parentheses_when_linting_then_ranking_rules_read_it()
-> Result<(), String> {
    let test_cases = [
        test_types::NamedWindowRankingTestCase {
            description: "a ranking sort above the cap in a WINDOW after the parentheses",
            sql: "(SELECT o.order_id, ROW_NUMBER() OVER w AS rn FROM orders AS o) WINDOW w AS (ORDER BY o.a, o.b, o.c, o.d, o.e, o.f)",
            rule: "SQLCLEAN043",
            expected_anchors: &["ROW_NUMBER"],
        },
        test_types::NamedWindowRankingTestCase {
            description: "the same with a WITH before the parentheses",
            sql: "WITH c AS (SELECT 1 AS x) (SELECT o.order_id, ROW_NUMBER() OVER w AS rn FROM orders AS o) WINDOW w AS (ORDER BY o.a, o.b, o.c, o.d, o.e, o.f)",
            rule: "SQLCLEAN043",
            expected_anchors: &["ROW_NUMBER"],
        },
        test_types::NamedWindowRankingTestCase {
            description: "an ordering without a declared key in a WINDOW after the parentheses is unproven",
            sql: "(SELECT o.order_id, ROW_NUMBER() OVER w AS rn FROM orders AS o) WINDOW w AS (ORDER BY o.ordered_at)",
            rule: "SQLCLEAN018",
            expected_anchors: &["ROW_NUMBER"],
        },
        test_types::NamedWindowRankingTestCase {
            description: "a ranking sort within the cap in a WINDOW after the parentheses is clean",
            sql: "(SELECT o.order_id, ROW_NUMBER() OVER w AS rn FROM orders AS o) WINDOW w AS (ORDER BY o.a)",
            rule: "SQLCLEAN043",
            expected_anchors: &[],
        },
    ];

    for test_case in &test_cases {
        let diagnostics =
            helpers::diagnostics_for_dialect(test_case.sql, "databricks", &[test_case.rule])?;
        let anchors: Vec<String> = diagnostics
            .iter()
            .map(|diagnostic| {
                let start = diagnostic["start"].as_u64().unwrap_or_default() as usize;
                let end = diagnostic["end"].as_u64().unwrap_or_default() as usize;
                test_case
                    .sql
                    .chars()
                    .skip(start)
                    .take(end - start)
                    .collect()
            })
            .collect();
        assert_eq!(
            anchors, test_case.expected_anchors,
            "{}",
            test_case.description
        );
    }
    Ok(())
}
