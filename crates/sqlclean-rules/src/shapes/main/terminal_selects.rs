//! `terminal_selects`: SQLCLEAN035's terminal steps that are more than a read of the last CTE.

use crate::models::{RuleFinding, RuleInput};
use crate::shapes::_helpers::terminals::{reads_last_cte, terminal};
use crate::trees::main::number_text::number_text;

const CEREMONIAL_SELECT: &str = "1";

/// After CTEs, the terminal step unless it lists columns of the last CTE alone.
pub fn terminal_selects(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    let Some(step) = terminal(input) else {
        return Vec::new();
    };
    let Some(with) = step.with else {
        return Vec::new();
    };
    let span = match (step.operator, step.select) {
        (Some(operator), _) => Some(operator),
        (None, Some(select)) if !reads_last_cte(select, with) => {
            let ceremonial: bool = input.settings.fixture.is_some()
                && select.from.is_none()
                && matches!(select.expressions.as_slice(), [item] if number_text(item) == Some(CEREMONIAL_SELECT));
            select.select_span.0.filter(|_| !ceremonial)
        }
        _ => None,
    };
    span.map(|span| RuleFinding { span, fix: None })
        .into_iter()
        .collect()
}
