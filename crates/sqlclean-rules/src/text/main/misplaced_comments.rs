//! `misplaced_comments`: SQLCLEAN033's comments, which the tree leaves out, against the keywords it records.

use sqlquack_syntax::Expression;
use sqlquack_syntax::tokens::models::Span;

use crate::models::{CommentPlacement, RuleFinding, RuleInput};
use crate::trees::main::statement_nodes::statement_nodes;

/// The comments that trail, are blocks, stand apart, or come before punctuation, THEN, ELSE or FROM.
pub fn misplaced_comments(
    input: &RuleInput<'_>,
    comments: &[CommentPlacement],
) -> Vec<RuleFinding> {
    if comments.is_empty() {
        return Vec::new();
    }
    let keywords: Vec<usize> = closing_keywords(input);
    comments
        .iter()
        .filter(|comment| {
            comment.trailing
                || comment.block
                || comment.detached
                || comment.before_punctuation
                || keywords.contains(&comment.anchor.start)
        })
        .map(|comment| RuleFinding {
            span: comment.anchor,
            fix: None,
        })
        .collect()
}

/// Where THEN, ELSE and FROM start.
fn closing_keywords(input: &RuleInput<'_>) -> Vec<usize> {
    let mut found: Vec<Span> = Vec::new();
    for node in statement_nodes(input) {
        match node {
            Expression::Case(case) => {
                let branches = (0..case.whens.len()).filter_map(|at| case.branch_keywords(at));
                found.extend(branches.map(|(_, then)| then));
                found.extend(case.else_keyword());
            }
            Expression::Select(select) => {
                found.extend(select.from.as_ref().and_then(|from| from.keyword_span.0));
            }
            Expression::Update(update) => {
                found.extend(
                    update
                        .from_clause
                        .as_ref()
                        .and_then(|from| from.keyword_span.0),
                );
            }
            _ => {}
        }
    }
    found.into_iter().map(|span| span.start).collect()
}
