//! The CROSS JOINs of a statement, each with the relation it starts from and the CTE it is in.

use sqlquack_syntax::expressions::queries::models::{Join, JoinKind};
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::JoinedName;

/// One CROSS JOIN.
pub(crate) struct CrossTarget<'a> {
    pub(crate) join: &'a Join,
    /// The relation before it, when it starts a chain.
    pub(crate) first: Option<&'a Expression>,
    pub(crate) cte: Option<&'a str>,
}

/// The CROSS JOINs of `statement`, from a worklist so long chains cannot overflow.
pub(crate) fn cross_targets(statement: &Expression) -> Vec<CrossTarget<'_>> {
    let mut pending: Vec<(&Expression, Option<&str>, bool)> = vec![(statement, None, true)];
    let mut found: Vec<CrossTarget<'_>> = Vec::new();
    while let Some((node, cte, top)) = pending.pop() {
        for (start, joins) in chains(node) {
            for (index, join) in joins.iter().enumerate() {
                if join.kind == JoinKind::Cross {
                    let first: Option<&Expression> = start.filter(|_| index == 0);
                    found.push(CrossTarget { join, first, cte });
                }
            }
        }
        let mut bodies: Vec<&Expression> = Vec::new();
        for definition in node.with_clause().iter().flat_map(|with| &with.ctes) {
            bodies.push(&definition.this);
            let label: Option<&str> = if top {
                Some(&definition.alias.name)
            } else {
                cte
            };
            pending.push((&definition.this, label, false));
        }
        let branches_are_top: bool = top && node.set_operation().is_some();
        for child in node.children() {
            if !bodies.iter().any(|body| std::ptr::eq(*body, child)) {
                pending.push((child, cte, branches_are_top));
            }
        }
    }
    found
}

fn chains(node: &Expression) -> Vec<(Option<&Expression>, &[Join])> {
    let lone = |items: &'_ [Expression]| items.len() == 1;
    match node {
        Expression::Select(select) => {
            let from = select.from.as_ref().filter(|from| lone(&from.expressions));
            vec![(
                from.map(|from| &from.expressions[0]),
                select.joins.as_slice(),
            )]
        }
        Expression::JoinedTable(joined) => vec![(Some(&joined.left), joined.joins.as_slice())],
        Expression::Update(update) => {
            let from = update
                .from_clause
                .as_ref()
                .filter(|from| lone(&from.expressions));
            vec![
                (
                    from.map(|from| &from.expressions[0]),
                    update.from_joins.as_slice(),
                ),
                (None, update.table_joins.as_slice()),
            ]
        }
        Expression::Delete(delete) => vec![(None, delete.joins.as_slice())],
        _ => Vec::new(),
    }
}

/// How a joined relation is named, when it is.
pub(crate) fn joined_relation_name(relation: &Expression) -> Option<JoinedName<'_>> {
    match relation {
        Expression::Table(table) => {
            if table.schema.is_none() && table.catalog.is_none() {
                return Some(JoinedName::Table(&table.name));
            }
            let mut parts: Vec<Option<_>> = [&table.catalog, &table.schema]
                .into_iter()
                .flatten()
                .chain([&table.name])
                .map(Some)
                .collect();
            if table.schema_omitted {
                parts.insert(1, None);
            }
            Some(JoinedName::Qualified(parts))
        }
        Expression::Subquery(subquery) => subquery
            .alias
            .as_ref()
            .map(|alias| JoinedName::Read(&alias.name)),
        Expression::Alias(alias) => Some(JoinedName::Alias(&alias.alias)),
        _ => None,
    }
}
