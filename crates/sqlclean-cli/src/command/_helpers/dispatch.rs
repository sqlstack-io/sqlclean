//! Argument parsing and the error boundary every subcommand shares.

use std::env;
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;

use crate::command::constants::{EXIT_ERROR, EXIT_SUCCESS};
use crate::command::models::{Cli, Command};
use crate::config::main::load_config::load_config;
use crate::config::models::Config;

pub(crate) fn run(arguments: Vec<OsString>) -> u8 {
    let cli: Cli = match Cli::try_parse_from(arguments) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return if error.use_stderr() {
                EXIT_ERROR
            } else {
                EXIT_SUCCESS
            };
        }
    };
    match execute(cli) {
        Ok(code) => code,
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "sqlclean: error: {error}");
            EXIT_ERROR
        }
    }
}

fn execute(cli: Cli) -> Result<u8, String> {
    if let Command::Facts(arguments) = &cli.command {
        return crate::command::_helpers::facts_command::run(arguments);
    }
    let start: PathBuf = env::current_dir().map_err(|error| error.to_string())?;
    let config: Config = load_config(cli.config.as_deref(), &start)?;
    match cli.command {
        Command::Check(arguments) => {
            crate::command::_helpers::check_command::run(&config, &arguments)
        }
        Command::Fix(arguments) => crate::command::_helpers::fix_command::run(&config, &arguments),
        Command::Format(arguments) => {
            crate::command::_helpers::format_command::run(&config, &arguments)
        }
        Command::Rules(arguments) => {
            crate::command::_helpers::rules_command::run(&config, &arguments)
        }
        Command::Skills(arguments) => {
            crate::command::_helpers::skills_command::run(&config.root, &arguments)
        }
        Command::Facts(_) => Ok(EXIT_SUCCESS),
    }
}
