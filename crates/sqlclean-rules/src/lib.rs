//! SQLClean's built-in Rules, which read SQLQuack's located tree and have no SQL text, tokens or parser.

#![forbid(unsafe_code)]

pub mod joins;
pub mod models;
pub mod set_operations;

pub use crate::joins::main::implicit_joins::implicit_joins;
pub use crate::joins::main::joins_without_condition::joins_without_condition;
pub use crate::set_operations::main::arity_mismatches::arity_mismatches;
pub use crate::set_operations::main::positional_stars::positional_stars;
