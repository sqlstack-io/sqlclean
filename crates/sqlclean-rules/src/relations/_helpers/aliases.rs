//! Relation aliases nothing reads.

use sqlquack_syntax::expressions::names::models::Identifier;
use sqlquack_syntax::expressions::queries::models::Select;
use sqlquack_syntax::tokens::models::Span;
use sqlquack_syntax::traversal::models::QueryScope;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{Replacement, RuleEdit, RuleFinding};
use crate::relations::_helpers::mentions::node_mentions;

/// One relation's alias as written.
struct WrittenAlias<'a> {
    name: &'a str,
    span: Span,
    /// The name the relation answers to without the alias.
    bare: Option<&'a str>,
    /// The alias is written before its relation, as `alias: relation`.
    prefix: bool,
}

/// The aliases of `scope` that nothing in its SELECT reads, with their removal where no names then collide.
pub(crate) fn unused_aliases(scope: &QueryScope<'_>, select: &Expression) -> Vec<RuleFinding> {
    let written: Vec<WrittenAlias<'_>> = written_aliases(scope);
    if written.is_empty() {
        return Vec::new();
    }
    let nodes: Vec<&Expression> = select.dfs().collect();
    let mut found: Vec<RuleFinding> = Vec::new();
    for alias in &written {
        if nodes.iter().any(|node| node_mentions(node, alias.name)) {
            continue;
        }
        let span: Span = explicit_span(scope.select, alias.span).unwrap_or(alias.span);
        let collides: bool = alias
            .bare
            .is_some_and(|bare| names_another(scope, (bare, alias.span)));
        let fix: Option<RuleEdit> = (!collides && !alias.prefix).then_some(RuleEdit {
            span,
            replacement: Replacement::Text(""),
            trailing_layout: false,
        });
        found.push(RuleFinding { span, fix });
    }
    found
}

fn written_aliases<'a>(scope: &QueryScope<'a>) -> Vec<WrittenAlias<'a>> {
    let mut found: Vec<WrittenAlias<'a>> = Vec::new();
    for relation in &scope.relations {
        let item: &'a Expression = relation.item;
        if has_column_aliases(item) {
            continue;
        }
        if let Expression::Lateral(lateral) = item {
            if let (Some(name), Some(whole)) = (&lateral.alias, lateral.span.0) {
                let quotes: usize = if lateral.alias_quoted { 2 } else { 0 };
                let length: usize = name.chars().count() + quotes;
                let span: Span = Span {
                    start: whole.end.saturating_sub(length),
                    ..whole
                };
                found.push(WrittenAlias {
                    name,
                    span,
                    bare: None,
                    prefix: false,
                });
            }
            continue;
        }
        let alias: Option<&'a Identifier> = relation.alias;
        if let Some(alias) = alias
            && let Some(span) = alias.span
        {
            let (bare, after): (Option<&'a str>, Option<Span>) = match item {
                Expression::Table(table) => (Some(&table.name.name), table.name.span),
                Expression::Subquery(query) => (None, query.this.source_span()),
                Expression::Alias(aliased) => (None, aliased.this.source_span()),
                _ => (None, None),
            };
            let prefix: bool = after.is_some_and(|body| span.start < body.start);
            found.push(WrittenAlias {
                name: &alias.name,
                span,
                bare,
                prefix,
            });
        }
    }
    found.extend(wrapped_aliases(scope.select));
    found
}

fn has_column_aliases(item: &Expression) -> bool {
    match item {
        Expression::Table(node) => !node.column_aliases.is_empty(),
        Expression::Subquery(node) => !node.column_aliases.is_empty(),
        Expression::Alias(node) => !node.column_aliases.is_empty(),
        Expression::Lateral(node) => !node.column_aliases.is_empty(),
        Expression::Pivot(node) => !node.alias_columns.is_empty(),
        _ => false,
    }
}

/// `AS alias`, when AS is written before the alias that ends where `alias` does.
fn explicit_span(select: &Select, alias: Span) -> Option<Span> {
    select
        .alias_spans
        .0
        .iter()
        .copied()
        .find(|written| written.end == alias.end)
}

/// Whether `name` is also what another relation of the scope answers to.
fn names_another(scope: &QueryScope<'_>, (name, alias): (&str, Span)) -> bool {
    scope.relations.iter().any(|relation| {
        let other: bool = relation
            .alias
            .and_then(|written| written.span)
            .is_none_or(|span| span.start != alias.start);
        let table: Option<&str> = match relation.item {
            Expression::Table(table) => Some(&table.name.name),
            _ => None,
        };
        other
            && (relation.name.eq_ignore_ascii_case(name)
                || table.is_some_and(|known| known.eq_ignore_ascii_case(name)))
    })
}

/// The aliases of derived tables that a sample or UNPIVOT wraps, which scopes leave out.
fn wrapped_aliases(select: &Select) -> Vec<WrittenAlias<'_>> {
    let items = select.from.iter().flat_map(|from| &from.expressions);
    let mut found: Vec<WrittenAlias<'_>> = Vec::new();
    for item in items.chain(select.joins.iter().map(|join| &join.this)) {
        let inner: Option<&Expression> = match item {
            Expression::TableSample(sampled) => sampled.this.as_deref(),
            Expression::Unpivot(unpivot) => Some(&unpivot.this),
            _ => None,
        };
        if let Some(Expression::Subquery(query)) = inner
            && let Some(alias) = &query.alias
            && let Some(span) = alias.span
        {
            found.push(WrittenAlias {
                name: &alias.name,
                span,
                bare: None,
                prefix: false,
            });
        }
    }
    found
}
