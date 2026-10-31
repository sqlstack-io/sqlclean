use crate::sql_facts::main::file_facts::file_facts;
use crate::sql_facts::main::statement_ends::statement_ends;
use crate::sql_facts::models::{FileFacts, StatementFacts};
use crate::sql_facts::tests::helpers::text;
use crate::sql_facts::tests::test_types::{
    ParseOutcomeTestCase, StatementEndsTestCase, StatementFactsTestCase,
};

const TWO_STATEMENTS: &str = "WITH recent AS (SELECT o.order_id, o.* FROM shop.orders AS o WHERE o.total > 10)\n\
SELECT r.order_id, UPPER(c.name) AS name, 'é' AS mark\n\
FROM recent AS r LEFT JOIN customers AS c ON r.customer_id = c.customer_id;\n\
SELECT COUNT(*) FROM customers";

#[test]
fn given_statements_when_collecting_facts_then_every_structure_is_located_in_characters()
-> Result<(), String> {
    let test_cases = [
        StatementFactsTestCase {
            description: "a CTE query after a multibyte literal locates every node by character",
            sql: TWO_STATEMENTS,
            statement: 0,
            expected_kind: "select",
            expected_text: "WITH recent AS (SELECT o.order_id, o.* FROM shop.orders AS o WHERE o.total > 10)\n\
SELECT r.order_id, UPPER(c.name) AS name, 'é' AS mark\n\
FROM recent AS r LEFT JOIN customers AS c ON r.customer_id = c.customer_id",
            expected_terminator: ";",
            expected_ctes: &["recent"],
            expected_tables: &[
                ("shop.orders", "o", "shop.orders"),
                ("recent", "r", "recent"),
                ("customers", "c", "customers"),
            ],
            expected_joins: &[(
                "LEFT",
                "customers",
                true,
                "LEFT JOIN customers AS c ON r.customer_id = c.customer_id",
            )],
            expected_stars: &["o.*"],
            expected_functions: &["UPPER"],
            expected_literals: &["10", "'é'"],
            expected_columns: &[("c", "customer_id"), ("o", "total")],
        },
        StatementFactsTestCase {
            description: "the last statement has no terminator and spans its own text",
            sql: TWO_STATEMENTS,
            statement: 1,
            expected_kind: "select",
            expected_text: "SELECT COUNT(*) FROM customers",
            expected_terminator: "",
            expected_ctes: &[],
            expected_tables: &[("customers", "", "customers")],
            expected_joins: &[],
            expected_stars: &[],
            expected_functions: &["COUNT"],
            expected_literals: &[],
            expected_columns: &[],
        },
        StatementFactsTestCase {
            description: "a join without a condition reports it",
            sql: "SELECT a.id FROM a CROSS JOIN b",
            statement: 0,
            expected_kind: "select",
            expected_text: "SELECT a.id FROM a CROSS JOIN b",
            expected_terminator: "",
            expected_ctes: &[],
            expected_tables: &[("a", "", "a"), ("b", "", "b")],
            expected_joins: &[("CROSS", "b", false, "CROSS JOIN b")],
            expected_stars: &[],
            expected_functions: &[],
            expected_literals: &[],
            expected_columns: &[("a", "id")],
        },
        StatementFactsTestCase {
            description: "join clauses end at the next join or clause, around subqueries and calls",
            sql: "SELECT a.id FROM a INNER JOIN (SELECT id FROM b) AS s ON a.id = s.id LEFT JOIN c USING (id) WHERE LEFT(a.name, 1) = 'x'",
            statement: 0,
            expected_kind: "select",
            expected_text: "SELECT a.id FROM a INNER JOIN (SELECT id FROM b) AS s ON a.id = s.id LEFT JOIN c USING (id) WHERE LEFT(a.name, 1) = 'x'",
            expected_terminator: "",
            expected_ctes: &[],
            expected_tables: &[("a", "", "a"), ("b", "", "b"), ("c", "", "c")],
            expected_joins: &[
                (
                    "INNER",
                    "",
                    true,
                    "INNER JOIN (SELECT id FROM b) AS s ON a.id = s.id",
                ),
                ("LEFT", "c", true, "LEFT JOIN c USING (id)"),
            ],
            expected_stars: &[],
            expected_functions: &["LEFT"],
            expected_literals: &["1", "'x'"],
            expected_columns: &[("a", "name"), ("s", "id")],
        },
    ];
    for test_case in &test_cases {
        let facts: FileFacts = file_facts(test_case.sql, "duckdb")?;
        let statement: &StatementFacts = &facts.statements[test_case.statement];
        let located = |start: usize, end: usize| text(test_case.sql, start, end);
        assert_eq!(
            statement.kind, test_case.expected_kind,
            "{}",
            test_case.description
        );
        assert_eq!(
            located(statement.start, statement.end),
            test_case.expected_text,
            "{}",
            test_case.description
        );
        assert_eq!(
            located(statement.end, statement.terminated_end),
            test_case.expected_terminator,
            "{}",
            test_case.description
        );
        let ctes: Vec<String> = statement
            .ctes
            .iter()
            .map(|cte| located(cte.start, cte.end))
            .collect();
        assert_eq!(ctes, test_case.expected_ctes, "{}", test_case.description);
        let tables: Vec<(&str, &str, String)> = statement
            .tables
            .iter()
            .map(|table| {
                (
                    table.qualified_name.as_str(),
                    table.alias.as_deref().unwrap_or(""),
                    located(table.start, table.end),
                )
            })
            .collect();
        let expected_tables: Vec<(&str, &str, String)> = test_case
            .expected_tables
            .iter()
            .map(|(name, alias, text)| (*name, *alias, (*text).to_owned()))
            .collect();
        assert_eq!(tables, expected_tables, "{}", test_case.description);
        let joins: Vec<(&str, &str, bool, String)> = statement
            .joins
            .iter()
            .map(|join| {
                (
                    join.kind.as_str(),
                    join.relation.as_deref().unwrap_or(""),
                    join.has_condition,
                    located(join.start, join.end),
                )
            })
            .collect();
        let expected_joins: Vec<(&str, &str, bool, String)> = test_case
            .expected_joins
            .iter()
            .map(|(kind, relation, condition, text)| {
                (*kind, *relation, *condition, (*text).to_owned())
            })
            .collect();
        assert_eq!(joins, expected_joins, "{}", test_case.description);
        let stars: Vec<String> = statement
            .stars
            .iter()
            .map(|star| located(star.start, star.end))
            .collect();
        assert_eq!(stars, test_case.expected_stars, "{}", test_case.description);
        let functions: Vec<String> = statement
            .functions
            .iter()
            .map(|function| located(function.start, function.end))
            .collect();
        assert_eq!(
            functions, test_case.expected_functions,
            "{}",
            test_case.description
        );
        let literals: Vec<String> = statement
            .literals
            .iter()
            .map(|literal| located(literal.start, literal.end))
            .collect();
        assert_eq!(
            literals, test_case.expected_literals,
            "{}",
            test_case.description
        );
        let columns: Vec<(&str, &str)> = statement
            .columns
            .iter()
            .map(|column| {
                (
                    column.qualifier.as_deref().unwrap_or(""),
                    column.name.as_str(),
                )
            })
            .collect();
        assert!(
            test_case
                .expected_columns
                .iter()
                .all(|expected| columns.contains(expected)),
            "{}: {columns:?}",
            test_case.description
        );
    }
    Ok(())
}

#[test]
fn given_sql_text_when_collecting_facts_then_only_unparseable_sql_is_an_error() {
    let test_cases = [
        ParseOutcomeTestCase {
            description: "a broken statement returns the parse error",
            sql: "SELECT FROM WHERE (",
            expected_statements: None,
        },
        ParseOutcomeTestCase {
            description: "two terminated statements both parse",
            sql: "SELECT 1; SELECT 2;",
            expected_statements: Some(2),
        },
    ];
    for test_case in &test_cases {
        let statements: Option<usize> = file_facts(test_case.sql, "duckdb")
            .map(|facts| facts.statements.len())
            .ok();
        assert_eq!(
            statements, test_case.expected_statements,
            "{}",
            test_case.description
        );
    }
}

#[test]
fn given_sql_text_when_finding_statement_ends_then_each_ends_after_its_terminator() {
    let test_cases = [
        StatementEndsTestCase {
            description: "a comment after a terminator starts the next statement's text",
            sql: "SELECT 'é;' AS a; -- next; note\nSELECT 2 /* end; */",
            expected_statements: Some(&["SELECT 'é;' AS a;", " -- next; note\nSELECT 2"]),
        },
        StatementEndsTestCase {
            description: "empty statements between semicolons are skipped",
            sql: ";; SELECT 1;;SELECT 2;",
            expected_statements: Some(&[";; SELECT 1;", ";SELECT 2;"]),
        },
        StatementEndsTestCase {
            description: "a file of only comments has no statements",
            sql: "-- only a note; nothing else\n/* still; nothing */\n",
            expected_statements: Some(&[]),
        },
        StatementEndsTestCase {
            description: "an unparseable statement is an error, not a split",
            sql: "SELECT 1; SELECT FROM WHERE (",
            expected_statements: None,
        },
    ];
    for test_case in &test_cases {
        let characters: Vec<char> = test_case.sql.chars().collect();
        let statements: Option<Vec<String>> = statement_ends(test_case.sql, "duckdb")
            .map(|ends| {
                let starts = std::iter::once(0).chain(ends.iter().copied());
                starts
                    .zip(ends.iter().copied())
                    .map(|(start, end)| characters[start..end].iter().collect())
                    .collect()
            })
            .ok();
        let texts: Option<Vec<&str>> = statements
            .as_ref()
            .map(|statements| statements.iter().map(String::as_str).collect());
        assert_eq!(
            texts.as_deref(),
            test_case.expected_statements,
            "{}",
            test_case.description
        );
    }
}
