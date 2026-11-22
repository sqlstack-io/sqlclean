//! `bare_unions`: SQLCLEAN008's UNIONs without ALL or DISTINCT.

use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{RuleEdit, RuleFinding, RuleInput};

/// The UNIONs that state neither ALL nor DISTINCT, by name or by position, with DISTINCT as their fix.
pub fn bare_unions(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for node in input
        .statements
        .iter()
        .flat_map(|statement| statement.dfs())
    {
        let Expression::Union(union) = node else {
            continue;
        };
        if union.all || union.distinct {
            continue;
        }
        let Some(span) = union.operator_span.0 else {
            continue;
        };
        let fix: Option<RuleEdit> = input.dialect.union_distinct.then_some(RuleEdit {
            span,
            replacement: "UNION DISTINCT",
            trailing_layout: false,
        });
        found.push(RuleFinding { span, fix });
    }
    found
}
