//! The `sqlclean` command: configuration, discovery, checking, fixing, formatting and reports.

#![forbid(unsafe_code)]

pub mod caching;
pub mod checking;
pub mod command;
pub mod config;
pub mod discovery;
pub mod fixing;
pub mod formatting;
pub mod hosting;
pub mod positions;
pub mod reporting;
