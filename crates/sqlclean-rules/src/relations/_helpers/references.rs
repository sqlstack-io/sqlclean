//! The column references a scope makes outside its FROM clause.

use std::collections::HashSet;

use sqlquack_syntax::expressions::names::models::Column;
use sqlquack_syntax::expressions::queries::models::Select;
use sqlquack_syntax::tokens::models::Span;
use sqlquack_syntax::traversal::models::QueryScope;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::RuleInput;
use crate::relations::constants::{CONTEXT_VALUES, DATE_PART_CALLS, WILDCARD};
use crate::trees::main::is_generated::is_generated;
use crate::trees::main::unwrapped::unwrapped;

/// A scope's references outside FROM and the joins, by how they are qualified.
pub(crate) struct References<'a> {
    pub(crate) qualified: Vec<&'a Column>,
    pub(crate) unqualified: Vec<&'a Column>,
}

/// The references `scope` writes outside its FROM clause that stand for columns.
pub(crate) fn references<'a>(input: &RuleInput<'_>, scope: &QueryScope<'a>) -> References<'a> {
    let select: &Select = scope.select;
    let joined: HashSet<*const Column> = relation_nodes(select)
        .into_iter()
        .filter_map(|node| match node {
            Expression::Column(column) => Some(std::ptr::from_ref(&**column)),
            _ => None,
        })
        .collect();
    let names: NotColumns<'_> = not_columns(select);
    let mut found = References {
        qualified: Vec::new(),
        unqualified: Vec::new(),
    };
    for &column in &scope.columns {
        let address: *const Column = std::ptr::from_ref(column);
        if joined.contains(&address) || column.name.name == WILDCARD {
            continue;
        }
        match &column.table {
            Some(table) => {
                let parameter: bool = names.parameters.contains(&table.name.as_str());
                if !parameter
                    && !names.dotted.contains(&address)
                    && !is_generated(input, &table.name)
                {
                    found.qualified.push(column);
                }
            }
            None => {
                if is_column_reference(input, (&names, column)) && !is_row(scope, &column.name.name)
                {
                    found.unqualified.push(column);
                }
            }
        }
    }
    found
}

/// The name a joined item answers to.
pub(crate) fn joined_name(item: &Expression) -> Option<&str> {
    match unwrapped(item) {
        Expression::Table(table) => Some(&table.alias.as_ref().unwrap_or(&table.name).name),
        Expression::Subquery(query) => query.alias.as_ref().map(|alias| alias.name.as_str()),
        Expression::Alias(alias) => Some(&alias.alias.name),
        Expression::Lateral(lateral) => lateral.alias.as_deref(),
        Expression::JoinedTable(joined) => joined.alias.as_ref().map(|alias| alias.name.as_str()),
        Expression::Pivot(pivot) => match &pivot.alias {
            Some(alias) => Some(&alias.name),
            None => None,
        },
        _ => None,
    }
}

/// What in a SELECT is written like a column and is none.
struct NotColumns<'a> {
    dotted: HashSet<*const Column>,
    parameters: Vec<&'a str>,
    /// Each output alias and where it is written.
    aliases: Vec<(&'a str, Span)>,
}

fn not_columns(select: &Select) -> NotColumns<'_> {
    let mut found = NotColumns {
        dotted: HashSet::new(),
        parameters: Vec::new(),
        aliases: Vec::new(),
    };
    for item in &select.expressions {
        if let Expression::Alias(alias) = item
            && let Some(span) = alias.alias.span
        {
            found.aliases.push((&alias.alias.name, span));
        }
    }
    for node in clause_nodes(select) {
        let root: Option<&Expression> = match node {
            Expression::Dot(dot) => Some(&dot.this),
            Expression::JsonExtract(path) | Expression::JsonExtractScalar(path) => Some(&path.this),
            Expression::JSONExtract(path) => Some(&path.this),
            Expression::Lambda(lambda) => {
                found
                    .parameters
                    .extend(lambda.parameters.iter().map(|name| name.name.as_str()));
                None
            }
            Expression::Function(call) => {
                date_part_argument(&call.name).and_then(|at| call.args.get(at))
            }
            _ => None,
        };
        if let Some(Expression::Column(column)) = root {
            found.dotted.insert(std::ptr::from_ref(&**column));
        }
    }
    found
}

fn is_column_reference(input: &RuleInput<'_>, (names, column): (&NotColumns<'_>, &Column)) -> bool {
    let name: &str = &column.name.name;
    let same = |other: &str| other.eq_ignore_ascii_case(name);
    let Some(written) = column.name.span.or(column.span) else {
        return false;
    };
    let aliased_before: bool = names
        .aliases
        .iter()
        .any(|(alias, span)| span.start < written.start && same(alias));
    !names.dotted.contains(&std::ptr::from_ref(column))
        && !is_generated(input, name)
        && (column.name.quoted || !CONTEXT_VALUES.iter().any(|value| same(value)))
        && !input
            .settings
            .external_identifiers
            .contains(&name.to_ascii_lowercase())
        && !names.parameters.iter().any(|parameter| same(parameter))
        && !aliased_before
}

/// Whether `name` alone is an alias in FROM, and so stands for that relation's row.
fn is_row(scope: &QueryScope<'_>, name: &str) -> bool {
    scope
        .relations
        .iter()
        .filter_map(|relation| relation.alias)
        .any(|alias| alias.name.eq_ignore_ascii_case(name))
}

/// Every node of FROM and the joins.
fn relation_nodes(select: &Select) -> Vec<&Expression> {
    let mut roots: Vec<&Expression> = select
        .from
        .iter()
        .flat_map(|from| &from.expressions)
        .collect();
    for join in &select.joins {
        roots.push(&join.this);
        roots.extend(join.on.as_ref());
        roots.extend(join.match_condition.as_ref());
        roots.extend(join.pivots.iter());
    }
    roots.into_iter().flat_map(|root| root.dfs()).collect()
}

/// Every node of the clauses after and before FROM.
pub(crate) fn clause_nodes(select: &Select) -> Vec<&Expression> {
    let mut roots: Vec<&Expression> = select.expressions.iter().collect();
    roots.extend(select.distinct_on.iter().flatten());
    roots.extend(select.lateral_views.iter().map(|view| &view.this));
    roots.extend(select.where_clause.iter().map(|clause| &clause.this));
    roots.extend(select.group_by.iter().flat_map(|group| &group.expressions));
    roots.extend(select.having.iter().map(|clause| &clause.this));
    roots.extend(select.qualify.iter().map(|clause| &clause.this));
    for window in select.windows.iter().flatten() {
        roots.extend(&window.spec.partition_by);
        roots.extend(window.spec.order_by.iter().map(|item| &item.this));
    }
    roots.extend(
        select
            .order_by
            .iter()
            .flat_map(|order| &order.expressions)
            .map(|item| &item.this),
    );
    roots.into_iter().flat_map(|root| root.dfs()).collect()
}

/// The position of the argument of call `name` that may be a date part written as a bare word.
fn date_part_argument(name: &str) -> Option<usize> {
    DATE_PART_CALLS
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(name))
        .map(|(_, position)| *position)
}
