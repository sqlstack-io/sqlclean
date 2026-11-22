//! `natural_joins`: SQLCLEAN051's NATURAL joins.

use sqlquack_syntax::expressions::queries::models::{Join, JoinKind};
use sqlquack_syntax::tokens::models::Span;

use crate::models::{RuleFinding, RuleInput};
use crate::trees::main::node_joins::node_joins;
use crate::trees::main::statement_nodes::statement_nodes;

/// Every NATURAL join, from NATURAL through JOIN.
pub fn natural_joins(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .flat_map(node_joins)
        .filter_map(natural_keywords)
        .map(|span| RuleFinding { span, fix: None })
        .collect()
}

fn natural_keywords(join: &Join) -> Option<Span> {
    let natural: bool = matches!(
        join.kind,
        JoinKind::Natural
            | JoinKind::NaturalLeft
            | JoinKind::NaturalRight
            | JoinKind::NaturalFull
            | JoinKind::NaturalSemi
            | JoinKind::NaturalAnti
    );
    Some(join.kind_span.0?.through(join.keyword_span.0?)).filter(|_| natural)
}
