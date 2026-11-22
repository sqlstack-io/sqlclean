//! `misnamed_final_ctes`: SQLCLEAN041's CTEs named against the `final` convention.

use sqlquack_syntax::expressions::queries::models::With;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::principal_query::principal_query;

const FINAL: &str = "final";

/// A query statement's last CTE not named final, and any CTE named final that is not last at the root.
pub fn misnamed_final_ctes(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    if input.settings.fixture.is_some() {
        return Vec::new();
    }
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        let body: Option<&With> = principal_query(statement).and_then(Expression::with_clause);
        let heads_statement: Option<&With> = statement.with_clause().or(body.filter(|_| {
            matches!(
                statement,
                Expression::Subquery(_) | Expression::Paren(_) | Expression::Annotated(_)
            )
        }));
        let roots: Vec<&With> = [heads_statement, body].into_iter().flatten().collect();
        for with in statement.dfs().filter_map(Expression::with_clause) {
            let at_root: bool = roots.iter().any(|root| std::ptr::eq(*root, with));
            let required: bool = heads_statement.is_some_and(|head| std::ptr::eq(head, with));
            for (index, cte) in with.ctes.iter().enumerate() {
                let last: bool = index + 1 == with.ctes.len();
                let named_final: bool = cte.alias.name.eq_ignore_ascii_case(FINAL);
                let misnamed: bool = match named_final {
                    true => !(last && at_root),
                    false => last && required,
                };
                if misnamed {
                    found.extend(cte.alias.span.map(|span| RuleFinding { span, fix: None }));
                }
            }
        }
    }
    found
}
