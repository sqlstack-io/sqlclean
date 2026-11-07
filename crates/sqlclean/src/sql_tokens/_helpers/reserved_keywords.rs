//! Per-dialect reserved keywords, which can never be unquoted identifiers in that dialect.

use std::collections::HashSet;
use std::sync::LazyLock;

use sqlquack_fingerprint::FingerprintDialect;

static BIGQUERY: LazyLock<HashSet<String>> =
    LazyLock::new(|| keyword_set(include_str!("../reserved_keywords/bigquery.txt")));
static TSQL: LazyLock<HashSet<String>> =
    LazyLock::new(|| keyword_set(include_str!("../reserved_keywords/tsql.txt")));
static DATABRICKS: LazyLock<HashSet<String>> =
    LazyLock::new(|| keyword_set(include_str!("../reserved_keywords/databricks.txt")));

/// Whether the upper-cased word is reserved in `dialect`; unlisted dialects use SQLQuack's generic list.
pub(crate) fn is_reserved_keyword(dialect: &str, upper_word: &str) -> bool {
    let keywords: &HashSet<String> = match dialect {
        "bigquery" => &BIGQUERY,
        "tsql" => &TSQL,
        "databricks" => &DATABRICKS,
        _ => {
            let fingerprint =
                FingerprintDialect::from_name(dialect).unwrap_or(FingerprintDialect::Generic);
            return sqlquack_fingerprint::is_reserved_keyword(fingerprint, upper_word);
        }
    };
    keywords.contains(upper_word)
}

fn keyword_set(data: &str) -> HashSet<String> {
    data.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_ascii_uppercase)
        .collect()
}
