use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlquack_format::{FormatDialect, FormatOptions};
use sqlquack_syntax::Dialect as SqlQuackDialect;

use crate::constants::INTERPOLATION_PLACEHOLDER_PREFIX;
use crate::sql_lint::_helpers::cte_macros::{take_out_cte_macros, wrap_query_placeholders};
use crate::sql_lint::constants::{
    BIGQUERY_DIALECT, DUCKDB_DIALECT, LINT_API_VERSION, POSTGRES_DIALECTS, SNOWFLAKE_DIALECT,
    TSQL_DIALECTS,
};
use crate::sql_lint::models::{FormatRequest, FormatResponse};

/// Spaces per indent step when a request does not choose.
const DEFAULT_INDENT: usize = 2;
const NOT_IDEMPOTENT_FAILURE: &str = "native formatter output is not idempotent";

pub(crate) fn format_json_impl(request_json: &str) -> Result<String, String> {
    let request: FormatRequest =
        serde_json::from_str(request_json).map_err(|error| error.to_string())?;
    let response = format_sql(request);
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

/// Format requests in parallel, in order; a request that fails is declined with its reason.
pub(crate) fn format_requests(requests: Vec<FormatRequest>) -> Vec<FormatResponse> {
    requests.into_par_iter().map(format_sql).collect()
}

/// Lay out `request`, or return its SQL unchanged with why it was declined.
fn format_sql(request: FormatRequest) -> FormatResponse {
    let declined =
        |sql: String, reason: Option<String>, unavailable: Option<String>| FormatResponse {
            version: LINT_API_VERSION,
            sql,
            changed: false,
            formatted: false,
            reason,
            unavailable,
        };
    if request.version != LINT_API_VERSION {
        let reason = format!(
            "unsupported native format request version {}; expected {LINT_API_VERSION}",
            request.version
        );
        return declined(request.sql, Some(reason), None);
    }
    let Some((dialect, macros)) = sqlquack_dialect(&request.dialect) else {
        let unavailable = format!(
            "SQL formatting is unavailable for the {} dialect: SQLQuack does not format it yet",
            request.dialect
        );
        return declined(request.sql, None, Some(unavailable));
    };
    let original = request.sql.clone();
    match format_with_sqlquack(request, dialect, macros) {
        Ok(response) => response,
        Err(reason) => declined(original, Some(reason), None),
    }
}

/// The SQLQuack dialect that lays out `dialect`, for the dialects SQLQuack formats natively, with the shared tokenizer's dialect that takes out CTE macros where it reads that dialect.
fn sqlquack_dialect(dialect: &str) -> Option<(FormatDialect, Option<SqlQuackDialect>)> {
    match dialect.to_ascii_lowercase().as_str() {
        SNOWFLAKE_DIALECT => Some((FormatDialect::Snowflake, Some(SqlQuackDialect::Snowflake))),
        DUCKDB_DIALECT => Some((FormatDialect::DuckDB, Some(SqlQuackDialect::DuckDB))),
        BIGQUERY_DIALECT => Some((FormatDialect::BigQuery, None)),
        name if POSTGRES_DIALECTS.contains(&name) => {
            Some((FormatDialect::PostgreSQL, Some(SqlQuackDialect::PostgreSQL)))
        }
        name if TSQL_DIALECTS.contains(&name) => Some((FormatDialect::SqlServer, None)),
        _ => None,
    }
}

/// Lay out `request` with SQLQuack; this re-checks idempotency only where placeholders were put back.
fn format_with_sqlquack(
    request: FormatRequest,
    dialect: FormatDialect,
    macros: Option<SqlQuackDialect>,
) -> Result<FormatResponse, String> {
    let options = FormatOptions {
        line_width: request.line_width,
        indent: request.indent.unwrap_or(DEFAULT_INDENT),
        token_widths: request.token_widths,
        parse: sqlquack_syntax::ParseOptions {
            complexity_guard: Some(sqlquack_syntax::ComplexityGuardOptions {
                max_function_call_depth: request.max_function_call_depth,
                ..Default::default()
            }),
        },
        ..FormatOptions::new(dialect)
    };
    let sql = format_around_cte_macros(&request.sql, macros, &options)?;
    if request.sql.contains(INTERPOLATION_PLACEHOLDER_PREFIX)
        && sql != request.sql
        && format_around_cte_macros(&sql, macros, &options)? != sql
    {
        return Err(NOT_IDEMPOTENT_FAILURE.to_string());
    }
    Ok(FormatResponse {
        version: LINT_API_VERSION,
        changed: sql != request.sql,
        sql,
        formatted: true,
        reason: None,
        unavailable: None,
    })
}

/// SQLQuack's layout of `sql`, which it checks for token identity and idempotency.
fn sqlquack_layout(sql: &str, options: &FormatOptions) -> Result<String, String> {
    sqlquack_format::format_sql(sql, options)
        .map(|formatted| formatted.sql)
        .map_err(|refusal| refusal.reason)
}

/// Lay out `sql` with SQLQuack, first making placeholders it cannot parse parseable and then restoring them, where `macros` names the shared tokenizer's dialect: a placeholder standing for a whole query is wrapped as `SELECT <placeholder>`, and CTE macro placeholders are taken out.
fn format_around_cte_macros(
    sql: &str,
    macros: Option<SqlQuackDialect>,
    options: &FormatOptions,
) -> Result<String, String> {
    let Some(dialect) = macros else {
        return sqlquack_layout(sql, options);
    };
    match wrap_query_placeholders(sql, dialect) {
        None => format_around_cte_lists(sql, dialect, options),
        Some(wrapped) => wrapped.restore(
            &format_around_cte_lists(&wrapped.sql, dialect, options)?,
            dialect,
        ),
    }
}

/// Lay out `sql` with SQLQuack, taking CTE macro placeholders out first and putting them back.
fn format_around_cte_lists(
    sql: &str,
    dialect: SqlQuackDialect,
    options: &FormatOptions,
) -> Result<String, String> {
    match take_out_cte_macros(sql, dialect)? {
        None => sqlquack_layout(sql, options),
        Some(without) => without.put_back(&sqlquack_layout(&without.sql, options)?, dialect),
    }
}
