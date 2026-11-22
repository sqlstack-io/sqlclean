//! `count_ones`: SQLCLEAN017's counts of the constant 1.

use sqlquack_syntax::Expression;
use sqlquack_syntax::tokens::models::Span;

use crate::calls::constants::COUNT;
use crate::calls::constants::ONE;
use crate::models::{Replacement, RuleEdit, RuleFinding, RuleInput};
use crate::trees::main::number_text::number_text;
use crate::trees::main::statement_nodes::statement_nodes;

/// The `1` of every COUNT(1), with `*` as the fix.
pub fn count_ones(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .filter_map(counted_one)
        .map(|span| RuleFinding {
            span,
            fix: Some(RuleEdit {
                span,
                replacement: Replacement::Text("*"),
                trailing_layout: false,
            }),
        })
        .collect()
}

fn counted_one(node: &Expression) -> Option<Span> {
    let (name, arguments, argument, distinct) = match node {
        Expression::Function(call) if call.qualified_name.len() <= 1 => (
            &call.name,
            &call.args,
            call.argument_span,
            call.set_quantifier.is_distinct(),
        ),
        Expression::AggregateFunction(call) if call.qualified_name.len() <= 1 => (
            &call.name,
            &call.args,
            call.argument_span,
            call.set_quantifier.is_distinct(),
        ),
        _ => return None,
    };
    let [value] = arguments.as_slice() else {
        return None;
    };
    let counts_one: bool =
        !distinct && name.eq_ignore_ascii_case(COUNT) && number_text(value) == Some(ONE);
    argument.0.filter(|_| counts_one)
}
