//! SQL lint Rules and the deterministic formatter; lint runs on SQLQuack, the formatter still on Polyglot.

pub mod _helpers;
pub mod constants;
pub mod main;
pub mod models;
pub mod types;

#[cfg(test)]
mod tests;
