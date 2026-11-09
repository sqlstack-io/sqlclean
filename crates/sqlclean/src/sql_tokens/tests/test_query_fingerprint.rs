use crate::sql_tokens::main::query_fingerprint::query_fingerprint;
use crate::sql_tokens::tests::test_types;

/// SQLBuild's stored golden fingerprints, which SQLClean must reproduce exactly.
#[test]
fn given_sqlbuild_golden_queries_when_fingerprinting_then_hashes_match_sqlbuild()
-> Result<(), String> {
    let test_cases = [
        test_types::FingerprintGoldenTestCase {
            description: "generic token stream",
            dialect: "generic",
            sql: "SELECT id, name FROM orders",
            expected_fingerprint: "264bcb1f81ce5ba25394a052016edc265aefba85c0698f616d3037f9d872d2bd",
        },
        test_types::FingerprintGoldenTestCase {
            description: "DuckDB dollar quote and cast",
            dialect: "duckdb",
            sql: "SELECT o.id, $$Customer's note$$ AS note, amount::DECIMAL(10, 2) AS amount FROM orders AS o",
            expected_fingerprint: "07786643d9593498ac2b2394e6801cb578b23ba0f9a0e70dbcdaf0d3a3cbf9db",
        },
        test_types::FingerprintGoldenTestCase {
            description: "PostgreSQL tagged dollar quote",
            dialect: "postgresql",
            sql: "SELECT o.id, $tag$it's$tag$ AS note, amount::NUMERIC AS amount FROM orders AS o",
            expected_fingerprint: "8db865c6f239b990c917754df176bd3c9b2c0c322b9af1437298ec98b67241f9",
        },
        test_types::FingerprintGoldenTestCase {
            description: "Snowflake path access and QUALIFY",
            dialect: "snowflake",
            sql: "SELECT o.id, payload:customer.name::STRING AS customer FROM orders AS o QUALIFY ROW_NUMBER() OVER (PARTITION BY o.id ORDER BY o.id) = 1",
            expected_fingerprint: "c75f7b5931a18e1a074ffa7ab113c256fc838285af75a3f9feb62d61cc76d94d",
        },
        test_types::FingerprintGoldenTestCase {
            description: "DuckDB lower-case keywords and SQLBuild placeholder",
            dialect: "duckdb",
            sql: "select o.id from __ref(\"orders\") as o where o.id > 0",
            expected_fingerprint: "fda88b9b0f26486bdd25b8e3bab5a982fbb387901ab229091b17772d044aebd0",
        },
        test_types::FingerprintGoldenTestCase {
            description: "BigQuery raw string and backtick path, as generic tokens",
            dialect: "bigquery",
            sql: "SELECT o.id, r'raw\\d' AS pattern FROM `project.dataset.orders` AS o",
            expected_fingerprint: "ce893f2fd9ab11ba453fc3c02e77b6690df6c3d1d8bc141fc19cf3891d9a61b4",
        },
        test_types::FingerprintGoldenTestCase {
            description: "Databricks optimizer hint, as generic tokens",
            dialect: "databricks",
            sql: "SELECT /*+ BROADCAST(o) */ o.id FROM `catalog`.`orders` AS o",
            expected_fingerprint: "7cd96f42bcb764136a7f423181302ba13cead02bb6cfb7382502a3a2aa4107c3",
        },
        // D651, D695: SQL Server is tokenized natively, so a bracketed name such as `[o]` is one
        // token. This is SQLBuild 0.204.0's digest; SQLClean 0.5.0 and 0.6.0 gave the generic
        // tokens' f1525ee2… instead.
        test_types::FingerprintGoldenTestCase {
            description: "SQL Server bracket identifiers, as SQL Server tokens",
            dialect: "sqlserver",
            sql: "SELECT TOP 10 [o].[id], N'order' AS kind FROM [dbo].[orders] AS o",
            expected_fingerprint: "f4fac3edb7682bbeddf4f73cc199493d45f1419bc8e2cd1d972fead2d67e6b71",
        },
    ];
    for test_case in &test_cases {
        assert_eq!(
            query_fingerprint(test_case.sql, test_case.dialect)?,
            test_case.expected_fingerprint,
            "{}",
            test_case.description
        );
    }
    Ok(())
}

/// Which texts and dialect names share a fingerprint: call names and dialect aliases ignore case.
#[test]
fn given_two_queries_when_fingerprinting_then_only_case_insensitive_differences_match()
-> Result<(), String> {
    let test_cases = [
        test_types::FingerprintEqualityTestCase {
            description: "an unquoted call name's case is ignored",
            left_dialect: "duckdb",
            left: "SELECT MyFunc(id) FROM orders",
            right_dialect: "duckdb",
            right: "SELECT myfunc(id) FROM orders",
            expected_equal: true,
        },
        test_types::FingerprintEqualityTestCase {
            description: "a quoted call name's case is significant",
            left_dialect: "snowflake",
            left: "SELECT \"MyFunc\"(id) FROM orders",
            right_dialect: "snowflake",
            right: "SELECT \"myfunc\"(id) FROM orders",
            expected_equal: false,
        },
        test_types::FingerprintEqualityTestCase {
            description: "a SQLBuild placeholder's case is significant",
            left_dialect: "duckdb",
            left: "SELECT id FROM __ref('orders')",
            right_dialect: "duckdb",
            right: "SELECT id FROM __REF('orders')",
            expected_equal: false,
        },
        test_types::FingerprintEqualityTestCase {
            description: "a CTE name with a column list is not a call",
            left_dialect: "duckdb",
            left: "WITH Tree(id) AS (SELECT 1) SELECT id FROM orders",
            right_dialect: "duckdb",
            right: "WITH tree(id) AS (SELECT 1) SELECT id FROM orders",
            expected_equal: false,
        },
        test_types::FingerprintEqualityTestCase {
            description: "an INSERT target with a column list is not a call",
            left_dialect: "postgres",
            left: "INSERT INTO Paid(id) SELECT 1",
            right_dialect: "postgres",
            right: "INSERT INTO paid(id) SELECT 1",
            expected_equal: false,
        },
        test_types::FingerprintEqualityTestCase {
            description: "a SQLite call name's case is ignored",
            left_dialect: "sqlite",
            left: "SELECT Upper(name) FROM orders",
            right_dialect: "sqlite",
            right: "SELECT UPPER(name) FROM orders",
            expected_equal: true,
        },
        test_types::FingerprintEqualityTestCase {
            description: "MotherDuck fingerprints as DuckDB",
            left_dialect: "motherduck",
            left: "select a from t qualify b",
            right_dialect: "duckdb",
            right: "select a from t qualify b",
            expected_equal: true,
        },
        test_types::FingerprintEqualityTestCase {
            description: "DuckDB reserves QUALIFY and generic SQL does not",
            left_dialect: "duckdb",
            left: "select a from t qualify b",
            right_dialect: "generic",
            right: "select a from t qualify b",
            expected_equal: false,
        },
    ];
    for test_case in &test_cases {
        assert_eq!(
            query_fingerprint(test_case.left, test_case.left_dialect)?
                == query_fingerprint(test_case.right, test_case.right_dialect)?,
            test_case.expected_equal,
            "{}",
            test_case.description
        );
    }
    Ok(())
}
