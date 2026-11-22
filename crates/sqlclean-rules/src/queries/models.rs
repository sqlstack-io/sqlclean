//! What the readings of a tree return.

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::names::models::Star;

/// One projection of a select list.
#[derive(Debug)]
pub(crate) enum Projection<'a> {
    Named {
        name: String,
        expression: &'a Expression,
    },
    Star(&'a Star),
    Unnamed,
}
