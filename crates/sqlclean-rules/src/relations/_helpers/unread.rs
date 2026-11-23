//! Joins whose relation nothing reads.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::{Join, JoinKind, Select};
use sqlquack_syntax::traversal::models::QueryScope;

use crate::models::RuleFinding;
use crate::relations::_helpers::mentions::{join_mentions, mentions, node_mentions};
use crate::relations::_helpers::references::{clause_nodes, joined_name};
use crate::trees::main::join_chains::join_chains;

pub(crate) fn projects_bare_star(scope: &QueryScope<'_>) -> bool {
    scope
        .select
        .expressions
        .iter()
        .any(|item| matches!(item, Expression::Star(star) if star.table.is_none()))
}

pub(crate) fn unread_joins(scope: &QueryScope<'_>) -> Vec<RuleFinding> {
    let select = scope.select;
    let clauses: Vec<&Expression> = clause_nodes(select);
    let joins: Vec<&Join> = join_chains(select)
        .into_iter()
        .flat_map(|(_, chain)| chain)
        .collect();
    let mut found: Vec<RuleFinding> = Vec::new();
    for (index, join) in joins.iter().enumerate() {
        if join.kind == JoinKind::Implicit {
            continue;
        }
        let (Some(name), Some(span)) = (joined_name(&join.this), join.keyword_span.0) else {
            continue;
        };
        let read_later: bool = joins[index + 1..]
            .iter()
            .any(|later| join_mentions(later, name))
            || lateral_mentions(select, name);
        if !read_later && !clauses.iter().any(|node| node_mentions(node, name)) {
            found.push(RuleFinding { span, fix: None });
        }
    }
    found
}

/// Whether a FROM item that may read the items before it could refer to `name`.
fn lateral_mentions(select: &Select, name: &str) -> bool {
    let mut items = select.from.iter().flat_map(|from| &from.expressions);
    items.any(|item| reads_earlier(item) && mentions(item, name))
}

fn reads_earlier(item: &Expression) -> bool {
    match item {
        Expression::Lateral(_) | Expression::Function(_) | Expression::Unnest(_) => true,
        Expression::Subquery(query) => query.lateral,
        Expression::Alias(alias) => {
            matches!(alias.this, Expression::Function(_) | Expression::Unnest(_))
        }
        _ => false,
    }
}
