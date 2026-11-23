//! `carried_statement`: the statement another explains or prepares.

use sqlquack_syntax::Expression;

/// The statement `statement` explains, describes or prepares, whose WITH is as much at the top as its own.
pub(crate) fn carried_statement(statement: &Expression) -> Option<&Expression> {
    match statement {
        Expression::UtilityStatement(utility) => utility.operands.last(),
        Expression::Prepare(prepare) => Some(&prepare.statement),
        Expression::Describe(described) => Some(&described.target),
        _ => None,
    }
}
