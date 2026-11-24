//! Unqualified references that name no single relation's column.

use sqlquack_syntax::expressions::names::models::Column;
use sqlquack_syntax::expressions::queries::models::Select;

use crate::models::RuleInput;
use crate::trees::main::join_chains::join_chains;

/// The columns the joins of `select` merge with USING, which no relation's alias can name alone.
pub(crate) fn using_columns(select: &Select) -> Vec<&str> {
    join_chains(select)
        .into_iter()
        .flat_map(|(_, joins)| joins)
        .flat_map(|join| &join.using)
        .map(|name| name.name.as_str())
        .collect()
}

/// Whether `column` reads one of the `merged` USING columns.
pub(crate) fn is_merged(merged: &[&str], column: &Column) -> bool {
    merged
        .iter()
        .any(|name| name.eq_ignore_ascii_case(&column.name.name))
}

/// Whether `column` is a macro parameter, which stands for what the caller passes.
pub(crate) fn is_macro_parameter(input: &RuleInput<'_>, column: &Column) -> bool {
    column
        .name
        .name
        .to_ascii_lowercase()
        .starts_with(input.settings.placeholders.macro_parameter)
}
