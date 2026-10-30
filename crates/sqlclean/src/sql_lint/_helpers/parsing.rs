//! The one tokenize-and-parse step every lint and fact request shares.

use std::str::FromStr;

use polyglot_sql::parser::ParserConfig;

use crate::sql_lint::models::ParsedSql;
use polyglot_sql::{
    ComplexityGuardOptions, Dialect, DialectType, Expression, ExpressionWalk, Parser,
};

const MAX_LINT_FUNCTION_CALL_DEPTH: usize = 128;

/// Tokenize and parse `sql`, rejecting statements the parser kept as opaque text.
pub(crate) fn parse(sql: &str, dialect_name: &str) -> Result<ParsedSql, String> {
    let dialect_type = DialectType::from_str(dialect_name).map_err(|error| error.to_string())?;
    let dialect = Dialect::get(dialect_type);
    let tokens = dialect.tokenize(sql).map_err(|error| error.to_string())?;
    let mut parser = Parser::with_config(
        tokens.clone(),
        ParserConfig {
            dialect: Some(dialect_type),
            complexity_guard: ComplexityGuardOptions {
                max_function_call_depth: Some(MAX_LINT_FUNCTION_CALL_DEPTH),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let statements = parser.parse().map_err(|error| error.to_string())?;
    for statement in &statements {
        if statement
            .dfs()
            .any(|node| matches!(node, Expression::Command(_) | Expression::Raw(_)))
        {
            return Err("unsupported syntax: native parser retained an opaque SQL node".to_owned());
        }
    }
    Ok(ParsedSql {
        dialect: dialect_type,
        tokens,
        statements,
    })
}
