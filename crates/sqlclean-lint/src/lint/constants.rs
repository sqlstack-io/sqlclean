/// Dialect names SQLClean's engine accepted that SQLQuack does not parse yet; their lint is unavailable.
pub(crate) const UNPARSED_DIALECTS: [&str; 38] = [
    "arrow-datafusion",
    "arrow_datafusion",
    "athena",
    "clickhouse",
    "cockroach",
    "cockroachdb",
    "datafusion",
    "doris",
    "dremio",
    "drill",
    "druid",
    "dune",
    "exasol",
    "fabric",
    "hana",
    "hive",
    "materialize",
    "memsql",
    "mssql",
    "mysql",
    "oracle",
    "presto",
    "redshift",
    "risingwave",
    "sap_hana",
    "saphana",
    "singlestore",
    "solr",
    "spark",
    "spark2",
    "sqlserver",
    "starrocks",
    "tableau",
    "teradata",
    "tidb",
    "trino",
    "tsql",
    "vertica",
];

pub const LINT_API_VERSION: u32 = 1;
/// The parts of a `SQLCLEAN038` message that names its joined relation and CTE.
pub(crate) const CROSS_JOIN_PREFIX: &str = "CROSS JOIN with `";
pub(crate) const CROSS_JOIN_CTE_MARKER: &str = "` in CTE `";
pub(crate) const CROSS_JOIN_SUFFIX: &str = "` creates an explicit cartesian product";
/// The `SQLCLEAN038` message for a join whose relation cannot be named.
pub(crate) const CROSS_JOIN_UNNAMED_MESSAGE: &str = "CROSS JOIN creates an explicit cartesian product with an unnamed relation; only a file-wide exception (rule and path) can cover it";
/// Stands for the finding's project-relative path until the caller that knows it fills it in.
pub(crate) const CROSS_JOIN_PATH_PLACEHOLDER: &str = "<path of this file>";
pub(crate) const CROSS_JOIN_REASON_PLACEHOLDER: &str = "<why this deviation is intentional>";
/// Why SQL in a dialect the crate was built without, through its feature, is not linted.
pub const UNBUILT_DIALECT_FAILURE: &str =
    "SQLQuack's lint was built without this dialect's feature";
