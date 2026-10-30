//! Walk each parsed statement once and collect its structural facts.

use polyglot_sql::expressions::{Column, Cte, Join, TableRef, With};
use polyglot_sql::tokens::{Span, Token, TokenType};
use polyglot_sql::{Expression, ExpressionWalk};

use crate::sql_facts::_helpers::statements::{StatementTokens, statement_token_ranges};
use crate::sql_facts::_helpers::tokens::{
    function_nodes, join_clause_span, literal_nodes, star_nodes,
};
use crate::sql_facts::models::{
    ColumnNode, FileFacts, JoinNode, NamedNode, StatementFacts, TableNode,
};
use crate::sql_lint::main::parse_sql::parse_sql;
use crate::sql_lint::models::ParsedSql;

pub(crate) fn file_facts(sql: &str, dialect: &str) -> Result<FileFacts, String> {
    let ParsedSql {
        tokens, statements, ..
    } = parse_sql(sql, dialect)?;
    let characters: Vec<char> = sql.chars().collect();
    let ranges: Vec<StatementTokens> = statement_token_ranges(&tokens);
    if ranges.len() != statements.len() {
        return Err(format!(
            "found {} statements but {} statement ranges",
            statements.len(),
            ranges.len()
        ));
    }
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
                },
                &characters,
            )
        })
        .collect();
    Ok(FileFacts { statements: facts })
}

/// One parsed statement, its tokens and the end offset of its terminator.
struct StatementSource<'a> {
    index: usize,
    statement: &'a Expression,
    tokens: &'a [Token],
    terminated_end: usize,
}

fn statement_facts(source: &StatementSource<'_>, characters: &[char]) -> StatementFacts {
    let StatementSource {
        index,
        statement,
        tokens,
        terminated_end,
    } = *source;
    let nodes: Vec<&Expression> = statement_nodes(statement);
    StatementFacts {
        index,
        kind: statement.variant_name().to_owned(),
        start: tokens.first().map_or(0, |token| token.span.start),
        end: tokens.last().map_or(0, |token| token.span.end),
        terminated_end,
        ctes: sorted(
            nodes
                .iter()
                .flat_map(|node| with_ctes(node))
                .filter_map(|cte| cte_node(cte, tokens))
                .collect(),
        ),
        tables: sorted(nodes.iter().filter_map(|node| table_node(node)).collect()),
        joins: sorted(
            nodes
                .iter()
                .flat_map(|node| select_joins(node))
                .filter_map(|join| join_node(join, tokens))
                .collect(),
        ),
        stars: star_nodes(tokens),
        functions: function_nodes(tokens),
        columns: sorted(nodes.iter().filter_map(|node| column_node(node)).collect()),
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
fn cte_node(cte: &Cte, tokens: &[Token]) -> Option<NamedNode> {
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
        name: cte.alias.name.clone(),
        start: span.start,
        end: span.end,
    })
}

fn table_node(node: &Expression) -> Option<TableNode> {
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
        .map(|part| part.name.as_str())
        .chain([table.name.name.as_str()])
        .collect::<Vec<&str>>()
        .join(".");
    Some(TableNode {
        start,
        end: span.end,
        name: table.name.name.clone(),
        qualified_name,
        alias: table.alias.as_ref().map(|alias| alias.name.clone()),
    })
}

fn join_node(join: &Join, tokens: &[Token]) -> Option<JoinNode> {
    let relation: Option<&TableRef> = match &join.this {
        Expression::Table(table) => Some(table),
        _ => None,
    };
    let (start, end) = join_clause_span(tokens, first_span(&join.this)?.start)?;
    Some(JoinNode {
        start,
        end,
        kind: format!("{:?}", join.kind).to_ascii_uppercase(),
        relation: relation.map(|table| table.name.name.clone()),
        has_condition: join.on.is_some() || !join.using.is_empty(),
    })
}

fn column_node(node: &Expression) -> Option<ColumnNode> {
    let Expression::Column(column) = node else {
        return None;
    };
    let column: &Column = column;
    let span: Span = column.span.or(column.name.span)?;
    Some(ColumnNode {
        start: span.start,
        end: span.end,
        name: column.name.name.clone(),
        qualifier: column.table.as_ref().map(|table| table.name.clone()),
    })
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
