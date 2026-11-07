use crate::sql_tokens::main::query_fingerprint::query_fingerprint;

/// SQLBuild's golden fingerprints for the dialects SQLQuack tokenizes; SQLBuild stores these
/// hashes, so SQLClean must compute the same ones.
#[test]
fn given_sqlbuild_golden_queries_when_fingerprinting_then_hashes_match_sqlbuild() -> Result<(), String>
{
    let test_cases: [(&str, &str, &str); 5] = [
        (
            "generic",
            "SELECT id, name FROM orders",
            "264bcb1f81ce5ba25394a052016edc265aefba85c0698f616d3037f9d872d2bd",
        ),
        (
            "duckdb",
            "SELECT o.id, $$Customer's note$$ AS note, amount::DECIMAL(10, 2) AS amount FROM orders AS o",
            "07786643d9593498ac2b2394e6801cb578b23ba0f9a0e70dbcdaf0d3a3cbf9db",
        ),
        (
            "postgresql",
            "SELECT o.id, $tag$it's$tag$ AS note, amount::NUMERIC AS amount FROM orders AS o",
            "8db865c6f239b990c917754df176bd3c9b2c0c322b9af1437298ec98b67241f9",
        ),
        (
            "snowflake",
            "SELECT o.id, payload:customer.name::STRING AS customer FROM orders AS o QUALIFY ROW_NUMBER() OVER (PARTITION BY o.id ORDER BY o.id) = 1",
            "c75f7b5931a18e1a074ffa7ab113c256fc838285af75a3f9feb62d61cc76d94d",
        ),
        (
            "duckdb",
            "select o.id from __ref(\"orders\") as o where o.id > 0",
            "fda88b9b0f26486bdd25b8e3bab5a982fbb387901ab229091b17772d044aebd0",
        ),
    ];
    for (dialect, sql, expected) in test_cases {
        assert_eq!(query_fingerprint(sql, dialect)?, expected, "{dialect}: {sql}");
    }
    Ok(())
}

/// Called names fold to upper case, except placeholders, quoted names and relation or CTE column lists.
#[test]
fn given_called_names_when_fingerprinting_then_only_unquoted_calls_ignore_case() -> Result<(), String>
{
    let test_cases: [(&str, &str, &str, bool); 6] = [
        ("duckdb", "SELECT MyFunc(id) FROM orders", "SELECT myfunc(id) FROM orders", true),
        ("snowflake", "SELECT \"MyFunc\"(id) FROM orders", "SELECT \"myfunc\"(id) FROM orders", false),
        ("duckdb", "SELECT id FROM __ref('orders')", "SELECT id FROM __REF('orders')", false),
        (
            "duckdb",
            "WITH Tree(id) AS (SELECT 1) SELECT id FROM orders",
            "WITH tree(id) AS (SELECT 1) SELECT id FROM orders",
            false,
        ),
        ("postgres", "INSERT INTO Paid(id) SELECT 1", "INSERT INTO paid(id) SELECT 1", false),
        ("sqlite", "SELECT Upper(name) FROM orders", "SELECT UPPER(name) FROM orders", true),
    ];
    for (dialect, left, right, expected_equal) in test_cases {
        assert_eq!(
            query_fingerprint(left, dialect)? == query_fingerprint(right, dialect)?,
            expected_equal,
            "{dialect}: {left} | {right}"
        );
    }
    Ok(())
}
