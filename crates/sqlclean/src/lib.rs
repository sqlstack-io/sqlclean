//! SQLClean's engine: lint and quality Rules, statement facts and the formatter, over Polyglot.

#![forbid(unsafe_code)]

pub mod constants;
pub mod sql_facts;
pub mod sql_lint;
pub mod sql_quality;
pub mod sql_tokens;
