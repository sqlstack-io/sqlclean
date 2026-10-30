//! Run `sqlclean` with the process arguments.

use std::process::ExitCode;

/// Parse the process arguments, run one subcommand and return its exit code.
pub fn run() -> ExitCode {
    ExitCode::from(crate::command::_helpers::dispatch::run(
        std::env::args_os().collect(),
    ))
}
