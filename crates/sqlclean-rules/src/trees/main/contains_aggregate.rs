//! `contains_aggregate`: whether a node or any node below it is an aggregate call.

use crate::models::DialectFacts;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::trees::main::is_aggregate::is_aggregate;

/// Whether `expression` or any node below it is an aggregate call.
pub(crate) fn contains_aggregate(expression: &Expression, dialect: &DialectFacts) -> bool {
    expression.contains(|node| is_aggregate(node, dialect))
}
