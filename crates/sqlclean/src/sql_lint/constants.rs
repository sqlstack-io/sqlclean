pub const LINT_API_VERSION: u32 = 1;
/// Nested function calls a request accepts by default.
pub const MAX_FUNCTION_CALL_DEPTH: usize =
    sqlquack_parse::parsing::constants::MAX_FUNCTION_CALL_DEPTH;
/// The most nested function calls a request may allow.
pub const MAX_CALL_DEPTH_CEILING: usize =
    sqlquack_parse::parsing::constants::MAX_CALL_DEPTH_CEILING;
/// The dialect name SQLQuack lays out as Snowflake.
pub(crate) const SNOWFLAKE_DIALECT: &str = "snowflake";
/// The dialect name SQLQuack lays out as DuckDB.
pub(crate) const DUCKDB_DIALECT: &str = "duckdb";
/// The dialect name SQLQuack lays out as BigQuery.
pub(crate) const BIGQUERY_DIALECT: &str = "bigquery";
/// The dialect name SQLQuack lays out as Databricks.
pub(crate) const DATABRICKS_DIALECT: &str = "databricks";
/// The dialect names SQLQuack lays out as PostgreSQL.
pub(crate) const POSTGRES_DIALECTS: [&str; 2] = ["postgres", "postgresql"];
/// The dialect names SQLQuack lays out as SQL Server.
pub(crate) const TSQL_DIALECTS: [&str; 3] = ["tsql", "mssql", "sqlserver"];
