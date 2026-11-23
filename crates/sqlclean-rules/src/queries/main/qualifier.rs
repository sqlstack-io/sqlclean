//! `qualifier`: the relation a column names.

use sqlquack_syntax::expressions::names::models::Column;

use crate::queries::main::lower::lower;

/// The qualifier of `column`, in lower case.
pub(crate) fn qualifier(column: &Column) -> Option<String> {
    column.table.as_ref().map(|table| lower(&table.name))
}
