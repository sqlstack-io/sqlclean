//! The statements of trigger programs, which Rules read as statements of their own.

use std::borrow::Cow;

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::schema_objects::models::TriggerBody;

/// `statements`, followed by the statements of each trigger program among them; borrowed when there are none.
pub(super) fn with_trigger_programs(statements: &[Expression]) -> Cow<'_, [Expression]> {
    let programs: Vec<&Expression> = statements.iter().flat_map(program).collect();
    if programs.is_empty() {
        return Cow::Borrowed(statements);
    }
    let mut all: Vec<Expression> = statements.iter().map(without_program).collect();
    all.extend(programs.into_iter().cloned());
    Cow::Owned(all)
}

/// `statement` with its trigger program taken out, so no Rule reads those statements twice.
fn without_program(statement: &Expression) -> Expression {
    let mut kept: Expression = statement.clone();
    if let Expression::CreateTrigger(trigger) = &mut kept
        && matches!(trigger.body, TriggerBody::Statements(_))
    {
        trigger.body = TriggerBody::Block(String::new());
    }
    kept
}

fn program(statement: &Expression) -> &[Expression] {
    match statement {
        Expression::CreateTrigger(trigger) => match &trigger.body {
            TriggerBody::Statements(statements) => statements,
            _ => &[],
        },
        _ => &[],
    }
}
