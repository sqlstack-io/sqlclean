//! `selects`: every SELECT of the statements a Rule is handed.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::queries::models::Select;

use crate::models::RuleInput;
use crate::trees::main::statement_nodes::statement_nodes;

/// Every SELECT of every statement, an outer one before those inside it.
pub(crate) fn selects<'a>(input: &RuleInput<'a>) -> impl Iterator<Item = &'a Select> {
    statement_nodes(input).filter_map(|node| match node {
        Expression::Select(select) => Some(&**select),
        _ => None,
    })
}
