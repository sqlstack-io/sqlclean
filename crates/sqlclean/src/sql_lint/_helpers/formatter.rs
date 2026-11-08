use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlquack_format::FormatOptions;
use sqlquack_syntax::Dialect as SqlQuackDialect;

use crate::constants::INTERPOLATION_PLACEHOLDER_PREFIX;
use crate::sql_lint::_helpers::cte_macros::take_out_cte_macros;
use crate::sql_lint::constants::{DUCKDB_DIALECT, LINT_API_VERSION, SNOWFLAKE_DIALECT};
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
    let Some(dialect) = sqlquack_dialect(&request.dialect) else {
        let unavailable = format!(
            "SQL formatting is unavailable for the {} dialect: SQLQuack does not format it yet",
            request.dialect
        );
        return declined(request.sql, None, Some(unavailable));
    };
    let original = request.sql.clone();
    match format_with_sqlquack(request, dialect) {
        Ok(response) => response,
        Err(reason) => declined(original, Some(reason), None),
    }
}

/// The SQLQuack dialect that lays out `dialect`, for the dialects SQLQuack formats natively.
fn sqlquack_dialect(dialect: &str) -> Option<SqlQuackDialect> {
    match dialect.to_ascii_lowercase().as_str() {
        SNOWFLAKE_DIALECT => Some(SqlQuackDialect::Snowflake),
        DUCKDB_DIALECT => Some(SqlQuackDialect::DuckDB),
        _ => None,
    }
}

/// Lay out `request` with SQLQuack; this re-checks idempotency only where placeholders were put back.
fn format_with_sqlquack(
    request: FormatRequest,
    dialect: SqlQuackDialect,
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
    let sql = format_around_cte_macros(&request.sql, &options)?;
    if request.sql.contains(INTERPOLATION_PLACEHOLDER_PREFIX)
        && sql != request.sql
        && format_around_cte_macros(&sql, &options)? != sql
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

/// Lay out `sql` with SQLQuack, taking CTE macro placeholders out first and putting them back.
fn format_around_cte_macros(sql: &str, options: &FormatOptions) -> Result<String, String> {
    match take_out_cte_macros(sql, options.dialect)? {
        None => sqlquack_layout(sql, options),
        Some(without) => {
            without.put_back(&sqlquack_layout(&without.sql, options)?, options.dialect)
        }
    }
}
