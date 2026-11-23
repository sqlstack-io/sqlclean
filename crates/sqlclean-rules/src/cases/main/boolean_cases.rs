//! `boolean_cases`: SQLCLEAN030's CASEs that only turn a condition into TRUE or FALSE.

use sqlquack_syntax::tokens::models::Span;

use crate::cases::_helpers::expressions::{case_expressions, is_boolean};
use crate::models::{Replacement, RuleEdit, RuleFinding, RuleInput};

/// The searched CASEs that are TRUE in every branch and FALSE otherwise; one branch has a fix.
pub fn boolean_cases(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for case in case_expressions(input) {
        let converts: bool = case.operand.is_none()
            && !case.whens.is_empty()
            && case.whens.iter().all(|(_, value)| is_boolean(value, true))
            && case
                .else_
                .as_ref()
                .is_some_and(|value| is_boolean(value, false));
        let (true, Some(first), Some(last), Some((when, then))) = (
            converts,
            case.case_keyword(),
            case.end_keyword(),
            case.branch_keywords(0),
        ) else {
            continue;
        };
        let span: Span = first.through(last);
        let fix: Option<RuleEdit> = (case.whens.len() == 1).then_some(RuleEdit {
            span,
            replacement: Replacement::Wrapped {
                before: "COALESCE(",
                kept: Span {
                    start: when.end,
                    end: then.start,
                    ..when
                },
                after: ", FALSE)",
            },
            trailing_layout: false,
        });
        found.push(RuleFinding { span, fix });
    }
    found
}
