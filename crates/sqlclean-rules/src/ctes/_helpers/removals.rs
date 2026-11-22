//! The edit that removes one CTE.

use sqlquack_syntax::expressions::queries::models::{Cte, With};
use sqlquack_syntax::tokens::models::Span;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{Replacement, RuleEdit};

const HOST_NAME_PREFIX: &str = "__";

/// The removal of CTE `index` of `with`, only of an authored CTE that does no more than read.
pub(crate) fn removal(with: &With, index: usize) -> Option<RuleEdit> {
    let cte: &Cte = with.ctes.get(index)?;
    if cte.alias.name.starts_with(HOST_NAME_PREFIX)
        || cte.materialized.is_some()
        || !cte.key_expressions.is_empty()
        || !cte.alias_first
        || !only_reads(&cte.this)
    {
        return None;
    }
    let (span, trailing_layout): (Span, bool) =
        match (with.ctes.get(index + 1), index.checked_sub(1)) {
            (Some(next), _) => {
                let name: Span = cte.alias.span?;
                let following: Span = next.alias.span?;
                (
                    Span {
                        end: following.start,
                        ..name
                    },
                    false,
                )
            }
            (None, Some(before)) => {
                let comma: Span = *with.comma_spans.0.get(before)?;
                (comma.through(cte.span.0?), false)
            }
            (None, None) if !with.recursive && with.search.is_none() => {
                (with.keyword_span.0?.through(cte.span.0?), true)
            }
            (None, None) => return None,
        };
    Some(RuleEdit {
        span,
        replacement: Replacement::Text(""),
        trailing_layout,
    })
}

fn only_reads(body: &Expression) -> bool {
    let is_query: bool = matches!(body.first_branch(), Expression::Select(_));
    is_query
        && !body.dfs().any(|node| {
            matches!(
                node,
                Expression::Insert(_)
                    | Expression::Update(_)
                    | Expression::Delete(_)
                    | Expression::Merge(_)
            )
        })
}
