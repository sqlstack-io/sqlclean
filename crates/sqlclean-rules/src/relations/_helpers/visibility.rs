//! The names a qualifier may use.

use sqlquack_syntax::traversal::models::QueryScopes;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::relations::constants::{CONFLICT_ROW, RULE_ROWS};
use crate::trees::main::join_chains::join_chains;

/// The relation names visible to scope `index`, an aliased table's own and leading part included.
pub(crate) fn visible_names<'a>(scopes: &QueryScopes<'a>, index: usize) -> Vec<&'a str> {
    let mut names: Vec<&'a str> = Vec::new();
    let mut next: Option<usize> = Some(index);
    while let Some(scope) = next.and_then(|at| scopes.scopes.get(at)) {
        names.extend(scope.relations.iter().map(|relation| relation.name));
        let select = scope.select;
        let items = select
            .from
            .iter()
            .flat_map(|from| &from.expressions)
            .chain(select.joins.iter().map(|join| &join.this));
        for node in items.flat_map(shallow_nodes) {
            match node {
                Expression::Table(table) => {
                    names.push(&table.name.name);
                    let leading = table.catalog.as_ref().or(table.schema.as_ref());
                    names.extend(leading.map(|part| part.name.as_str()));
                    names.extend(table.alias.as_ref().map(|alias| alias.name.as_str()));
                }
                Expression::Lateral(lateral) => names.extend(lateral.alias.as_deref()),
                Expression::Alias(alias) => names.push(&alias.alias.name),
                Expression::Identifier(identifier) => names.push(&identifier.name),
                Expression::Pivot(pivot) => {
                    names.extend(pivot.alias.as_ref().map(|alias| alias.name.as_str()));
                }
                Expression::MatchRecognize(matched) => {
                    names.extend(matched.alias.as_ref().map(|alias| alias.name.as_str()));
                }
                _ => {}
            }
        }
        for join in join_chains(select).into_iter().flat_map(|(_, joins)| joins) {
            names.extend(join.using_alias.as_ref().map(|alias| alias.name.as_str()));
        }
        for view in &select.lateral_views {
            names.extend(view.table_alias.as_ref().map(|alias| alias.name.as_str()));
        }
        next = scope.outer;
    }
    names
}

/// The nodes of a FROM item outside the queries inside it.
fn shallow_nodes(item: &Expression) -> Vec<&Expression> {
    let mut found: Vec<&Expression> = Vec::new();
    let mut pending: Vec<&Expression> = vec![item];
    while let Some(node) = pending.pop() {
        found.push(node);
        if !matches!(node, Expression::Subquery(_) | Expression::Select(_)) {
            pending.extend(node.children());
        }
    }
    found
}

/// The row names `statement` itself puts in sight of every query inside it.
pub(crate) fn statement_rows(statement: &Expression) -> Vec<&'static str> {
    let mut node: &Expression = statement;
    if let Expression::UtilityStatement(utility) = statement
        && let Some(body) = utility.operands.last()
    {
        node = body;
    }
    match node {
        Expression::Insert(_) => vec![CONFLICT_ROW],
        Expression::CreateObject(_) => RULE_ROWS.to_vec(),
        _ => Vec::new(),
    }
}
