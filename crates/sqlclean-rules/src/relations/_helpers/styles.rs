//! Which references break a scope's style of qualification.

use sqlquack_syntax::expressions::names::models::Column;
use sqlquack_syntax::tokens::models::Span;

use crate::models::RuleInput;

/// The references to report: a passed relation's read in a macro's output takes neither style, and a plain slot read none.
pub(crate) fn inconsistent(
    input: &RuleInput<'_>,
    (qualified, unqualified): (&[&Column], &[&Column]),
) -> Vec<Span> {
    let outputs = input.settings.macro_outputs;
    let output_of = |column: &Column| -> Option<usize> {
        let start: usize = reference_start(column)?.start;
        outputs
            .iter()
            .position(|output| output.start <= start && start < output.end)
    };
    let mut all = qualified.iter().chain(unqualified);
    if let Some(first) = all.next()
        && let Some(output) = output_of(first)
        && all.all(|column| output_of(column) == Some(output))
    {
        return Vec::new();
    }
    let mut unstyled: Vec<&Column> = unqualified.to_vec();
    let mut styled: usize = 0;
    for &column in qualified {
        let Some(table) = &column.table else {
            continue;
        };
        let passed: bool = output_of(column).is_some_and(|output| {
            outputs[output]
                .relations
                .iter()
                .any(|relation| relation.eq_ignore_ascii_case(&table.name))
        });
        if passed {
            continue;
        }
        match is_parameter(input, &table.name) && is_parameter(input, &column.name.name) {
            true => unstyled.push(column),
            false => styled += 1,
        }
    }
    if styled == 0 {
        return Vec::new();
    }
    let mut spans: Vec<Span> = unstyled.into_iter().filter_map(reference_start).collect();
    spans.sort_by_key(|span| span.start);
    spans
}

fn reference_start(column: &Column) -> Option<Span> {
    match &column.table {
        Some(table) => table.span.or(column.span),
        None => column.name.span.or(column.span),
    }
}

fn is_parameter(input: &RuleInput<'_>, name: &str) -> bool {
    name.to_ascii_lowercase()
        .starts_with(input.settings.placeholders.macro_parameter)
}
