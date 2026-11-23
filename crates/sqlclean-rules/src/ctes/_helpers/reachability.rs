//! Which CTEs a query reaches.

use std::collections::{HashMap, HashSet};

use sqlquack_syntax::expressions::queries::models::{Cte, With};
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::RuleInput;

/// The WITH of a query, the only kind an unread CTE is judged in.
pub(crate) fn query_with(node: &Expression) -> Option<&With> {
    match node {
        Expression::Select(_)
        | Expression::Union(_)
        | Expression::Intersect(_)
        | Expression::Except(_) => node.with_clause(),
        _ => None,
    }
}

/// The indexes of `with`'s CTEs that `root` does not reach.
pub(crate) fn unreached(input: &RuleInput<'_>, (root, with): (&Expression, &With)) -> Vec<usize> {
    let names: Vec<String> = with
        .ctes
        .iter()
        .map(|cte: &Cte| cte.alias.name.to_ascii_lowercase())
        .collect();
    let mut outside: HashMap<String, usize> = HashMap::new();
    for name in read_names(root) {
        *outside.entry(name).or_default() += 1;
    }
    let inside = with
        .ctes
        .iter()
        .map(|cte| &cte.this)
        .chain(with.search.as_deref());
    for body in inside {
        for name in read_names(body) {
            if let Some(count) = outside.get_mut(&name) {
                *count = count.saturating_sub(1);
            }
        }
    }
    let mut reachable: HashSet<&str> = HashSet::new();
    let mut queue: Vec<&str> = Vec::new();
    for name in &names {
        let read_outside: bool = outside.get(name).is_some_and(|count| *count > 0);
        if read_outside || is_read_by_host(input, name) {
            reachable.insert(name);
            queue.push(name);
        }
    }
    let reads: Vec<HashSet<String>> = with
        .ctes
        .iter()
        .map(|cte| read_names(&cte.this).collect())
        .collect();
    while let Some(reader) = queue.pop() {
        for (name, read) in names.iter().zip(&reads) {
            if name != reader {
                continue;
            }
            for dependency in &names {
                if read.contains(dependency) && reachable.insert(dependency) {
                    queue.push(dependency);
                }
            }
        }
    }
    (0..names.len())
        .filter(|&index| !reachable.contains(names[index].as_str()))
        .collect()
}

fn is_read_by_host(input: &RuleInput<'_>, name: &str) -> bool {
    input.settings.externally_referenced_ctes.contains(name)
        || input.settings.fixture.is_some_and(|fixture| {
            fixture
                .cte_prefixes
                .iter()
                .any(|prefix| name.starts_with(prefix.as_str()))
        })
}

/// The unqualified table names read in `expression`: a qualified name is never a CTE.
fn read_names(expression: &Expression) -> impl Iterator<Item = String> + '_ {
    expression.dfs().filter_map(|node| match node {
        Expression::Table(table) if table.schema.is_none() && table.catalog.is_none() => {
            Some(table.name.name.to_ascii_lowercase())
        }
        _ => None,
    })
}
