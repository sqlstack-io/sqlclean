//! `repeated_names`: the names that repeat an earlier one.

use sqlquack_syntax::expressions::names::models::Identifier;

use crate::models::{DialectFacts, RuleFinding};
use crate::names::main::resolved::resolved;

/// A finding at each of `names` the dialect takes for the same as one before it.
pub(crate) fn repeated_names<'a>(
    dialect: &DialectFacts,
    names: impl IntoIterator<Item = &'a Identifier>,
) -> Vec<RuleFinding> {
    let mut earlier: Vec<String> = Vec::new();
    let mut found: Vec<RuleFinding> = Vec::new();
    for identifier in names {
        let name: String = resolved(dialect, identifier);
        if earlier
            .iter()
            .any(|known| (dialect.same_name)(known, &name))
        {
            found.extend(identifier.span.map(|span| RuleFinding { span, fix: None }));
        } else {
            earlier.push(name);
        }
    }
    found
}
