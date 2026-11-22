//! `unused_joined_relations`: SQLCLEAN032's joins that contribute no values.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::JoinKind;
use sqlquack_syntax::traversal::models::QueryScope;

use crate::models::{RuleFinding, RuleInput};
use crate::relations::_helpers::mentions::{mentions, node_mentions};
use crate::relations::_helpers::references::{clause_nodes, joined_name, references};

/// The JOINs whose relation nothing after them reads, where every reference says what it reads.
pub fn unused_joined_relations(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        for scope in statement.query_scopes().scopes {
            if scope.select.joins.is_empty() || projects_bare_star(&scope) {
                continue;
            }
            if references(input, &scope).unqualified.is_empty() {
                found.extend(unread_joins(&scope));
            }
        }
    }
    found
}

fn projects_bare_star(scope: &QueryScope<'_>) -> bool {
    scope
        .select
        .expressions
        .iter()
        .any(|item| matches!(item, Expression::Star(star) if star.table.is_none()))
}

fn unread_joins(scope: &QueryScope<'_>) -> Vec<RuleFinding> {
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
        let read_later: bool = select.joins[index + 1..].iter().any(|later| {
            std::iter::once(&later.this)
                .chain(later.on.as_ref())
                .chain(later.match_condition.as_ref())
                .any(|part| mentions(part, name))
        });
        if !read_later && !clauses.iter().any(|node| node_mentions(node, name)) {
            found.push(RuleFinding { span, fix: None });
        }
    }
    found
}
