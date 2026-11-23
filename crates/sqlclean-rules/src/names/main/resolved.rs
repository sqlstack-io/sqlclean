//! `resolved`: a name as the dialect resolves it.

use sqlquack_syntax::expressions::names::models::Identifier;

use crate::models::DialectFacts;

/// The name `identifier` resolves to.
pub(crate) fn resolved(dialect: &DialectFacts, identifier: &Identifier) -> String {
    (dialect.resolved_name)(&identifier.name, identifier.quoted)
}
