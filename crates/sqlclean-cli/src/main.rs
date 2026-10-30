//! The `sqlclean` executable.

use std::process::ExitCode;

use sqlclean_cli::command::main::run::run;

fn main() -> ExitCode {
    run()
}
