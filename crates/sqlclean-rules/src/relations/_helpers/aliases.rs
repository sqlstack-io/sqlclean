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
    /// The alias has no plain removal: it is written before its relation, as `alias: relation`, or its clause needs it.
    kept: bool,
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
        let fix: Option<RuleEdit> = (!collides && !alias.kept).then_some(RuleEdit {
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
            if let (Some(name), Some(span)) = (&lateral.alias, lateral.alias_span.0) {
                found.push(WrittenAlias {
                    name,
                    span,
                    bare: None,
                    kept: false,
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
                Expression::Subquery(query) => (None, query.span.0.or(query.this.source_span())),
                Expression::Alias(aliased) => (None, aliased.this.source_span()),
                _ => (None, None),
            };
            let kept: bool = after.is_some_and(|body| span.start < body.start);
            found.push(WrittenAlias {
                name: &alias.name,
                span,
                bare,
                kept,
            });
        }
    }
    for wrapped in wrapped_aliases(scope.select) {
        if !found.iter().any(|known| known.span == wrapped.span) {
            found.push(wrapped);
        }
    }
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

/// The aliases inside a sample or UNPIVOT and of lateral views, which scopes leave out.
fn wrapped_aliases(select: &Select) -> Vec<WrittenAlias<'_>> {
    let items = select.from.iter().flat_map(|from| &from.expressions);
    let mut found: Vec<WrittenAlias<'_>> = Vec::new();
    for item in items.chain(select.joins.iter().map(|join| &join.this)) {
        let inner: Option<&Expression> = match item {
            Expression::TableSample(sampled) => sampled.this.as_deref(),
            Expression::Unpivot(unpivot) => {
                found.extend(written(unpivot.alias.as_ref(), true));
                Some(&unpivot.this)
            }
            _ => None,
        };
        let alias: Option<&Identifier> = match inner {
            Some(Expression::Subquery(query)) => query.alias.as_ref(),
            Some(Expression::Table(table)) => table.alias.as_ref(),
            _ => None,
        };
        found.extend(written(alias, false));
    }
    for view in &select.lateral_views {
        found.extend(written(view.table_alias.as_ref(), true));
    }
    found
}

fn written(alias: Option<&Identifier>, kept: bool) -> Option<WrittenAlias<'_>> {
    let alias: &Identifier = alias?;
    Some(WrittenAlias {
        name: &alias.name,
        span: alias.span?,
        bare: None,
        kept,
    })
}
