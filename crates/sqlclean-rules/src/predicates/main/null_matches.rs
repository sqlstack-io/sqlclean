//! `null_matches`: SQLCLEAN047's matches against NULL.

use sqlquack_syntax::Expression;
use sqlquack_syntax::tokens::models::Span;

use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::statement_nodes::statement_nodes;
use crate::trees::main::unwrapped::unwrapped;

/// `CASE x WHEN NULL`, and IN over NULLs alone where NOT is not written before IN.
pub fn null_matches(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut negated: Vec<*const Expression> = Vec::new();
    let mut found: Vec<Span> = Vec::new();
    for node in statement_nodes(input) {
        match node {
            Expression::Not(negation) if !negation.operator_span.is_absent() => {
                negated.push(std::ptr::from_ref(&negation.this));
            }
            Expression::In(list)
                if !list.not
                    && list.query.is_none()
                    && !list.expressions.is_empty()
                    && list.expressions.iter().all(is_null)
                    && !negated.contains(&std::ptr::from_ref(node)) =>
            {
                found.extend(list.operator_span.operation());
            }
            Expression::Case(case) if case.operand.is_some() => {
                let tests = case.whens.iter().zip(&case.when_spans.0);
                found.extend(
                    tests
                        .filter(|((test, _), _)| is_null(test))
                        .map(|(_, span)| *span),
                );
            }
            _ => {}
        }
    }
    found
        .into_iter()
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}

fn is_null(value: &Expression) -> bool {
    matches!(unwrapped(value), Expression::Null(_))
}
