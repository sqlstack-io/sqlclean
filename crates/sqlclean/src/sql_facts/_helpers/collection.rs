//! Walk each parsed statement once and collect its structural facts.

use sqlquack_syntax::expressions::names::models::{Column, Identifier, TableRef};
use sqlquack_syntax::expressions::queries::models::{Cte, Join, With};
use sqlquack_syntax::tokens::models::{Span, Token, TokenType};
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::sql_facts::_helpers::statements::{StatementTokens, statement_token_ranges};
use crate::sql_facts::_helpers::tokens::{
    function_nodes, join_clause_span, literal_nodes, star_nodes,
};
use crate::sql_facts::constants::{
    ALTER_OBJECT_KINDS, CREATE_OBJECT_KINDS, DATE_PART_CALLS, DATE_PART_NAMES,
};
use crate::sql_facts::models::{
    ColumnNode, FileFacts, JoinNode, NamedNode, StatementFacts, TableNode,
};
use crate::sql_lint::constants::LINT_API_VERSION;
use crate::sql_lint::main::lint::lint;
use crate::sql_lint::main::parse_sql::parse_sql;
use crate::sql_lint::types::{LintRequest, LintResponse, ParsedSql};

pub(crate) fn file_facts(sql: &str, dialect: &str) -> Result<FileFacts, String> {
    if let Some(reason) = unavailable(dialect)? {
        return Ok(FileFacts {
            statements: Vec::new(),
            unavailable: Some(reason),
        });
    }
    let (
        ParsedSql {
            tokens, statements, ..
        },
        ranges,
    ) = parsed_statements(sql, dialect)?;
    let characters: Vec<char> = sql.chars().collect();
    let snowflake: bool = dialect.eq_ignore_ascii_case("snowflake");
    let facts: Vec<StatementFacts> = statements
        .iter()
        .zip(ranges)
        .enumerate()
        .map(|(index, (statement, range))| {
            statement_facts(
                &StatementSource {
                    index,
                    statement,
                    tokens: &tokens[range.first..=range.last],
                    terminated_end: range.terminated_end,
                    snowflake,
                },
                &characters,
            )
        })
        .collect();
    Ok(FileFacts {
        statements: facts,
        unavailable: None,
    })
}

pub(crate) fn statement_ends(sql: &str, dialect: &str) -> Result<Option<Vec<usize>>, String> {
    if unavailable(dialect)?.is_some() {
        return Ok(None);
    }
    let (_, ranges) = parsed_statements(sql, dialect)?;
    Ok(Some(
        ranges.iter().map(|range| range.terminated_end).collect(),
    ))
}

/// Why facts are unavailable for `dialect`: lint's own answer for a dialect SQLQuack does not parse yet.
fn unavailable(dialect: &str) -> Result<Option<String>, String> {
    let probe: LintResponse = lint(LintRequest {
        version: LINT_API_VERSION,
        dialect: dialect.to_owned(),
        enabled_rules: Some(Vec::new()),
        ..LintRequest::default()
    })?;
    Ok(probe.unavailable.map(|_| {
        format!(
            "SQL facts are unavailable for the {dialect} dialect: SQLQuack does not parse it yet"
        )
    }))
}

/// Parse `sql` and split its tokens into one range per parsed statement.
fn parsed_statements(
    sql: &str,
    dialect: &str,
) -> Result<(ParsedSql, Vec<StatementTokens>), String> {
    let parsed: ParsedSql = parse_sql(sql, dialect)?;
    let ranges: Vec<StatementTokens> = statement_token_ranges(&parsed.tokens);
    if ranges.len() != parsed.statements.len() {
        return Err(format!(
            "found {} statements but {} statement ranges",
            parsed.statements.len(),
            ranges.len()
        ));
    }
    Ok((parsed, ranges))
}

/// One parsed statement, its tokens and the end offset of its terminator.
struct StatementSource<'a> {
    index: usize,
    statement: &'a Expression,
    tokens: &'a [Token],
    terminated_end: usize,
    /// Whether the dialect is Snowflake, whose date functions take a bare date part.
    snowflake: bool,
}

fn statement_facts(source: &StatementSource<'_>, characters: &[char]) -> StatementFacts {
    let StatementSource {
        index,
        statement,
        tokens,
        terminated_end,
        snowflake,
    } = *source;
    let nodes: Vec<&Expression> = statement_nodes(statement);
    let date_parts: Vec<&Expression> = match snowflake {
        true => nodes
            .iter()
            .filter_map(|node| date_part_argument(node))
            .collect(),
        false => Vec::new(),
    };
    StatementFacts {
        index,
        kind: statement_kind(statement),
        start: tokens.first().map_or(0, |token| token.span.start),
        end: tokens.last().map_or(0, |token| token.span.end),
        terminated_end,
        ctes: sorted(
            nodes
                .iter()
                .flat_map(|node| with_ctes(node))
                .filter_map(|cte| cte_node(cte, tokens, characters))
                .collect(),
        ),
        tables: sorted(
            nodes
                .iter()
                .filter_map(|node| table_node(node, characters))
                .collect(),
        ),
        joins: sorted(
            nodes
                .iter()
                .flat_map(|node| select_joins(node))
                .filter_map(|join| join_node(join, tokens, characters))
                .collect(),
        ),
        stars: star_nodes(tokens),
        functions: function_nodes(tokens),
        columns: sorted(
            nodes
                .iter()
                .filter(|node| !is_date_part(node, &date_parts))
                .filter_map(|node| column_node(node, characters))
                .collect(),
        ),
        literals: literal_nodes(tokens, characters),
    }
}

/// Every node of `expression`, including the bodies of the CTEs its queries define.
fn statement_nodes(expression: &Expression) -> Vec<&Expression> {
    let mut nodes: Vec<&Expression> = Vec::new();
    for node in expression.dfs() {
        nodes.push(node);
        for cte in with_ctes(node) {
            nodes.extend(statement_nodes(&cte.this));
        }
    }
    nodes
}

fn sorted<T: Ord>(mut items: Vec<T>) -> Vec<T> {
    items.sort();
    items.dedup();
    items
}

fn with_ctes(node: &Expression) -> &[Cte] {
    let with: Option<&With> = match node {
        Expression::Select(select) => select.with.as_ref(),
        Expression::Union(union) => union.with.as_ref(),
        Expression::Intersect(intersect) => intersect.with.as_ref(),
        Expression::Except(except) => except.with.as_ref(),
        Expression::Insert(insert) => insert.with.as_ref(),
        Expression::Update(update) => update.with.as_ref(),
        Expression::Delete(delete) => delete.with.as_ref(),
        _ => None,
    };
    with.map_or(&[], |with| with.ctes.as_slice())
}

fn select_joins(node: &Expression) -> &[Join] {
    match node {
        Expression::Select(select) => select.joins.as_slice(),
        _ => &[],
    }
}

/// The CTE located at its name; the parser leaves CTE names unlocated, so read `name AS (`.
fn cte_node(cte: &Cte, tokens: &[Token], characters: &[char]) -> Option<NamedNode> {
    let span: Span = cte.alias.span.or_else(|| {
        tokens
            .windows(2)
            .find(|pair| {
                pair[1].token_type == TokenType::As
                    && pair[0]
                        .text
                        .trim_matches(['"', '`', '[', ']'])
                        .eq_ignore_ascii_case(&cte.alias.name)
            })
            .map(|pair| pair[0].span)
    })?;
    Some(NamedNode {
        name: authored(&cte.alias, characters),
        start: span.start,
        end: span.end,
    })
}

fn table_node(node: &Expression, characters: &[char]) -> Option<TableNode> {
    let Expression::Table(table) = node else {
        return None;
    };
    let span: Span = table.span.or(table.name.span)?;
    let start: usize = [&table.catalog, &table.schema]
        .into_iter()
        .flatten()
        .filter_map(|part| part.span)
        .map(|part| part.start)
        .fold(span.start, usize::min);
    let qualified_name: String = [&table.catalog, &table.schema]
        .into_iter()
        .flatten()
        .chain([&table.name])
        .map(|part| authored(part, characters))
        .collect::<Vec<String>>()
        .join(".");
    Some(TableNode {
        start,
        end: span.end,
        name: authored(&table.name, characters),
        qualified_name,
        alias: table
            .alias
            .as_ref()
            .map(|alias| authored(alias, characters)),
    })
}

fn join_node(join: &Join, tokens: &[Token], characters: &[char]) -> Option<JoinNode> {
    let relation: Option<&TableRef> = match &join.this {
        Expression::Table(table) => Some(table),
        _ => None,
    };
    let (start, end) = join_clause_span(tokens, first_span(&join.this)?.start)?;
    Some(JoinNode {
        start,
        end,
        kind: format!("{:?}", join.kind).to_ascii_uppercase(),
        relation: relation.map(|table| authored(&table.name, characters)),
        has_condition: join.on.is_some() || !join.using.is_empty(),
    })
}

/// A Snowflake date call's bare date part, such as `day` in `DATEPART(day, ts)`: a unit, as Polyglot read it.
fn date_part_argument(node: &Expression) -> Option<&Expression> {
    let Expression::Function(function) = node else {
        return None;
    };
    let (_, position, any_name) = DATE_PART_CALLS
        .iter()
        .find(|(name, _, _)| name.eq_ignore_ascii_case(&function.name))?;
    function
        .args
        .get(*position)
        .filter(|argument| reads_as_unit(argument, *any_name))
}

/// Whether `argument` is a bare name that reads as a date part: any name, or only a known unit.
fn reads_as_unit(argument: &Expression, any_name: bool) -> bool {
    match argument {
        Expression::Column(column) if column.table.is_none() && !column.name.quoted => {
            any_name || DATE_PART_NAMES.contains(&column.name.name.to_ascii_uppercase().as_str())
        }
        _ => false,
    }
}

/// Whether `node` is one of the date parts found in its statement.
fn is_date_part(node: &Expression, date_parts: &[&Expression]) -> bool {
    date_parts.iter().any(|part| std::ptr::eq(*part, node))
}

fn column_node(node: &Expression, characters: &[char]) -> Option<ColumnNode> {
    let Expression::Column(column) = node else {
        return None;
    };
    let column: &Column = column;
    let span: Span = column.span.or(column.name.span)?;
    let start: usize = column
        .table
        .as_ref()
        .and_then(|table| table.span)
        .map_or(span.start, |table| table.start.min(span.start));
    Some(ColumnNode {
        start,
        end: span.end,
        name: authored(&column.name, characters),
        qualifier: column
            .table
            .as_ref()
            .map(|table| authored(table, characters)),
    })
}

/// A name as written: a parser that folds unquoted names keeps the folded name, so read the source.
fn authored(identifier: &Identifier, characters: &[char]) -> String {
    let written: Option<String> = identifier
        .span
        .filter(|_| !identifier.quoted)
        .and_then(|span| characters.get(span.start..span.end))
        .map(|text| text.iter().collect());
    written
        .filter(|text| text.eq_ignore_ascii_case(&identifier.name))
        .unwrap_or_else(|| identifier.name.clone())
}

/// The statement's kind, naming a generic CREATE or ALTER by its object as Polyglot's tree did.
fn statement_kind(statement: &Expression) -> String {
    let (kind, known): (&str, &[(&str, &str)]) = match statement {
        Expression::CreateObject(node) => (&node.kind, &CREATE_OBJECT_KINDS),
        Expression::AlterObject(node) => (&node.kind, &ALTER_OBJECT_KINDS),
        _ => ("", &[]),
    };
    known
        .iter()
        .find(|(object, _)| object.eq_ignore_ascii_case(kind))
        .map_or(statement.variant_name(), |(_, name)| name)
        .to_owned()
}

/// The first located table or column inside `expression`, in source order.
fn first_span(expression: &Expression) -> Option<Span> {
    expression
        .dfs()
        .filter_map(|node| match node {
            Expression::Table(table) => table.span.or(table.name.span),
            Expression::Column(column) => column.span,
            _ => None,
        })
        .min_by_key(|span| span.start)
}
