//! `implicit_inner_joins`: SQLCLEAN025's JOINs that leave INNER unsaid.

use sqlquack_syntax::expressions::queries::models::{Join, JoinKind};

use crate::models::{Replacement, RuleEdit, RuleFinding, RuleInput};
use crate::trees::main::node_joins::node_joins;
use crate::trees::main::statement_nodes::statement_nodes;

/// The inner JOINs with a condition and no INNER, with it as the fix unless DIRECTED is between.
pub fn implicit_inner_joins(input: &RuleInput<'_>) -> Vec<RuleFinding> {
    statement_nodes(input)
        .flat_map(node_joins)
        .filter_map(implicit_inner)
        .collect()
}

fn implicit_inner(join: &Join) -> Option<RuleFinding> {
    let (first, keyword) = (join.kind_span.0?, join.keyword_span.0?);
    let conditioned: bool = join.on.is_some() || !join.using.is_empty();
    let unsaid: bool = join.kind == JoinKind::Inner
        && !join.use_inner_keyword
        && conditioned
        && (join.directed || first == keyword);
    let fix: Option<RuleEdit> = (!join.directed).then_some(RuleEdit {
        span: keyword,
        replacement: Replacement::Text("INNER JOIN"),
        trailing_layout: false,
    });
    unsaid.then_some(RuleFinding { span: keyword, fix })
}
