//! SQLClean's engine: lint and quality Rules, statement facts and the formatter, over SQLQuack for lint and facts; the formatter still reads Polyglot.

#![forbid(unsafe_code)]

pub mod constants;
pub mod sql_facts;
pub mod sql_lint;
pub mod sql_tokens;
