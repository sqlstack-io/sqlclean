//! `projections`: a select list, item by item.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::Select;

use crate::queries::main::lower::lower;
use crate::queries::models::Projection;

/// Each item of the select list.
pub(crate) fn projections(select: &Select) -> Vec<Projection<'_>> {
    select
        .expressions
        .iter()
        .map(|expression| match expression {
            Expression::Alias(alias) => Projection::Named {
                name: lower(&alias.alias.name),
                expression: &alias.this,
            },
            Expression::Column(column) => Projection::Named {
                name: lower(&column.name.name),
                expression,
            },
            Expression::Star(star) => Projection::Star(star),
            _ => Projection::Unnamed,
        })
        .collect()
}
