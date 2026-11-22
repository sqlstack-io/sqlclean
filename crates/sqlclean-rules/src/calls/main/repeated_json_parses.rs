//! `repeated_json_parses`: SQLCLEAN045's payloads parsed more than once in one SELECT.

use sqlquack_syntax::Expression;

use crate::calls::_helpers::json_parses::repeated_in_scope;
use crate::models::{JsonParseFinding, RuleInput};
use crate::trees::main::statement_nodes::statement_nodes;

/// The second parse of each JSON payload column parsed again in one SELECT.
pub fn repeated_json_parses<'a>(input: &RuleInput<'a>) -> Vec<JsonParseFinding<'a>> {
    if input.dialect.json_parse_functions.is_empty() {
        return Vec::new();
    }
    statement_nodes(input)
        .filter(|node| matches!(node, Expression::Select(_)))
        .flat_map(|select| repeated_in_scope(select, &input.dialect))
        .collect()
}
