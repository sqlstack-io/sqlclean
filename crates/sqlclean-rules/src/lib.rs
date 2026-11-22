//! SQLClean's built-in Rules, which read SQLQuack's located tree and have no SQL text, tokens or parser.

#![forbid(unsafe_code)]

pub mod cases;
pub mod joins;
pub mod models;
pub mod set_operations;
pub(crate) mod trees;

pub use crate::cases::main::boolean_cases::boolean_cases;
pub use crate::cases::main::nested_else_cases::nested_else_cases;
pub use crate::cases::main::redundant_else_nulls::redundant_else_nulls;
pub use crate::joins::main::implicit_joins::implicit_joins;
pub use crate::joins::main::joins_without_condition::joins_without_condition;
pub use crate::set_operations::main::arity_mismatches::arity_mismatches;
pub use crate::set_operations::main::bare_unions::bare_unions;
pub use crate::set_operations::main::positional_stars::positional_stars;
