//! `projected_stars`: SQLCLEAN021's uncontrolled select lists.

use sqlquack_syntax::expressions::queries::models::Select;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{RuleFinding, RuleInput};
use crate::selects::_helpers::star_exemptions::exempt_stars;
use crate::selects::_helpers::stars::{asterisk, is_wildcard};

/// The asterisks projected by any SELECT, other than those nobody can enumerate.
pub fn projected_stars(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let mut found: Vec<RuleFinding> = Vec::new();
    for statement in input.statements {
        let exempt: Vec<&Select> = exempt_stars(input, statement);
        for node in statement.dfs() {
            let Expression::Select(select) = node else {
                continue;
            };
            if exempt.iter().any(|kept| std::ptr::eq(*kept, &**select)) {
                continue;
            }
            for (item, written) in select.expressions.iter().zip(&select.item_spans.0) {
                if is_wildcard(item)
                    && let Some(written) = written.operation()
                    && let Some(span) = asterisk(item, written)
                {
                    found.push(RuleFinding { span, fix: None });
                }
            }
        }
    }
    found
}
