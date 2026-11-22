//! `sources`: the relations a SELECT reads.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::Select;

use crate::queries::main::lower::lower;

/// Relations read directly by one select: `(alias, relation name, expression)`.
pub(crate) fn sources(select: &Select) -> Vec<(String, Option<String>, &Expression)> {
    let mut found: Vec<(String, Option<String>, &Expression)> = Vec::new();
    let from = select.from.iter().flat_map(|from| from.expressions.iter());
    for expression in from
        .chain(select.joins.iter().map(|join| &join.this))
        .flat_map(joined_relations)
    {
        match expression {
            Expression::Table(table) => {
                let name: String = lower(&table.name.name);
                let alias: String = table
                    .alias
                    .as_ref()
                    .map_or_else(|| name.clone(), |alias| lower(&alias.name));
                found.push((alias, Some(name), expression));
            }
            Expression::Subquery(subquery) => {
                let alias: String = subquery
                    .alias
                    .as_ref()
                    .map_or_else(String::new, |alias| lower(&alias.name));
                found.push((alias, None, expression));
            }
            _ => found.push((String::new(), None, expression)),
        }
    }
    found
}

/// The relations of an unaliased joined table, in parentheses or not, in order; else `item`.
fn joined_relations(item: &Expression) -> Vec<&Expression> {
    match item {
        Expression::JoinedTable(joined) if joined.alias.is_none() => {
            let mut relations: Vec<&Expression> = joined_relations(&joined.left);
            for join in &joined.joins {
                relations.extend(joined_relations(&join.this));
            }
            relations
        }
        Expression::Paren(paren) if matches!(paren.this, Expression::JoinedTable(_)) => {
            joined_relations(&paren.this)
        }
        _ => vec![item],
    }
}
