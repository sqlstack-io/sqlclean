//! Joins whose relation nothing reads.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::JoinKind;
use sqlquack_syntax::traversal::models::QueryScope;

use crate::models::RuleFinding;
use crate::relations::_helpers::mentions::{join_mentions, node_mentions};
use crate::relations::_helpers::references::{clause_nodes, joined_name};

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
    let mut found: Vec<RuleFinding> = Vec::new();
    for (index, join) in select.joins.iter().enumerate() {
        if join.kind == JoinKind::Implicit {
            continue;
        }
        let (Some(name), Some(span)) = (joined_name(&join.this), join.keyword_span.0) else {
            continue;
        };
        let read_later: bool = select.joins[index + 1..]
            .iter()
            .any(|later| join_mentions(later, name));
        if !read_later && !clauses.iter().any(|node| node_mentions(node, name)) {
            found.push(RuleFinding { span, fix: None });
        }
    }
    found
}
