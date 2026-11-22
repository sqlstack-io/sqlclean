//! `nested_ctes`: SQLCLEAN036's WITHs inside a query.

use sqlquack_syntax::expressions::queries::models::With;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::carried_statement::carried_statement;
use crate::trees::main::principal_query::principal_query;

/// The WITH keywords that head neither a statement nor the query that is its body.
pub fn nested_ctes(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        let written: Option<&With> = statement.with_clause();
        let carried: Option<&With> = carried_statement(statement).and_then(Expression::with_clause);
        let body: Option<&With> = principal_query(statement).and_then(Expression::with_clause);
        for with in statement.dfs().filter_map(Expression::with_clause) {
            let at_root: bool = [written, carried, body]
                .into_iter()
                .flatten()
                .any(|root| std::ptr::eq(root, with));
            if !at_root {
                found.extend(
                    with.keyword_span
                        .0
                        .map(|span| RuleFinding { span, fix: None }),
                );
            }
        }
    }
    found
}
