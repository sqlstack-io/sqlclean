//! JSON parse calls whose argument resolves to a plain column.

use std::collections::HashMap;

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::names::models::Column;
use sqlquack_syntax::tokens::models::Span;

use crate::models::{DialectFacts, JsonParseFinding};
use crate::names::main::resolved::resolved;
use crate::trees::main::call_name::call_name;
use crate::trees::main::call_positions::call_positions;
use crate::trees::main::walk_in_scope::walk_in_scope;

/// One JSON parse call whose argument resolves to a plain column.
struct ParseCall<'a> {
    function: &'static str,
    column: &'a Column,
    column_start: usize,
    span: Span,
}

/// What two parses share when they parse one payload.
type Payload = (&'static str, Option<String>, String);

/// The second of each column's parses in `select`'s scope.
pub(crate) fn repeated_in_scope<'a>(
    select: &'a Expression,
    dialect: &DialectFacts,
) -> Vec<JsonParseFinding<'a>> {
    let mut calls: Vec<ParseCall<'a>> = walk_in_scope(select)
        .filter_map(|node| parse_call(node, dialect.json_parse_functions))
        .collect();
    calls.sort_by_key(|call| call.column_start);
    let key = |name| match (dialect.same_name)("a", "A") {
        true => resolved(dialect, name).to_ascii_lowercase(),
        false => resolved(dialect, name),
    };
    let mut groups: HashMap<Payload, Vec<&ParseCall<'a>>> = HashMap::new();
    for call in &calls {
        let payload: Payload = (
            call.function,
            call.column.table.as_ref().map(key),
            key(&call.column.name),
        );
        groups.entry(payload).or_default().push(call);
    }
    let mut found: Vec<JsonParseFinding<'a>> = Vec::new();
    for group in groups.values() {
        if let [_, second, ..] = group.as_slice() {
            found.push(JsonParseFinding {
                span: second.span,
                function: second.function,
                qualifier: second.column.table.as_ref(),
                column: &second.column.name,
                count: group.len(),
            });
        }
    }
    found
}

fn parse_call<'a>(node: &'a Expression, functions: &[&'static str]) -> Option<ParseCall<'a>> {
    let (name, argument): (&str, &Expression) = match node {
        Expression::ParseJson(function) => ("PARSE_JSON", &function.this),
        Expression::Function(function) if function.args.len() == 1 => {
            (function.name.as_str(), &function.args[0])
        }
        _ => return None,
    };
    let function: &'static str = functions
        .iter()
        .copied()
        .find(|candidate| candidate.eq_ignore_ascii_case(name))?;
    let column: &Column = unwrap_column(argument)?;
    let start: Span = column.span.or(column.name.span)?;
    let named: Option<Span> = call_positions(node).0.0;
    Some(ParseCall {
        function,
        column,
        column_start: start.start,
        span: named.unwrap_or(Span {
            end: start.start,
            ..start
        }),
    })
}

/// The column under wrappers that keep the parsed value tied to that column.
fn unwrap_column(expression: &Expression) -> Option<&Column> {
    let mut current: &Expression = expression;
    loop {
        current = match current {
            Expression::Column(column) => return Some(column),
            Expression::Paren(paren) => &paren.this,
            Expression::Lower(function) | Expression::Upper(function) => &function.this,
            Expression::Trim(trim) if trim.characters.is_none() => &trim.this,
            Expression::Cast(cast) | Expression::TryCast(cast) | Expression::SafeCast(cast) => {
                &cast.this
            }
            Expression::Coalesce(coalesce) if coalesce.expressions.len() == 1 => {
                &coalesce.expressions[0]
            }
            Expression::Function(function)
                if function.args.len() == 1
                    && call_name(current).is_some_and(|name| {
                        ["LOWER", "UPPER", "TRIM", "COALESCE"]
                            .iter()
                            .any(|wrapper| wrapper.eq_ignore_ascii_case(name))
                    }) =>
            {
                &function.args[0]
            }
            _ => return None,
        };
    }
}
