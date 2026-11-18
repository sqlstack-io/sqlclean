//! `sqlclean facts`: the statement facts of SQL on standard input, for Rule test harnesses.

use std::io::{Read, Write};

use sqlclean::sql_facts::main::file_facts::file_facts;
use sqlclean::sql_lint::constants::MAX_CALL_DEPTH_CEILING;

use crate::command::constants::EXIT_SUCCESS;
use crate::command::models::FactsArgs;

pub(crate) fn run(arguments: &FactsArgs) -> Result<u8, String> {
    if !(1..=MAX_CALL_DEPTH_CEILING).contains(&arguments.max_function_call_depth) {
        return Err(format!(
            "--max-function-call-depth must be between 1 and {MAX_CALL_DEPTH_CEILING}"
        ));
    }
    let mut sql = String::new();
    std::io::stdin()
        .read_to_string(&mut sql)
        .map_err(|error| error.to_string())?;
    let facts = file_facts(
        &sql,
        &arguments.dialect,
        Some(arguments.max_function_call_depth),
    )?;
    let json: String = serde_json::to_string(&facts).map_err(|error| error.to_string())?;
    let _ = writeln!(std::io::stdout(), "{json}");
    Ok(EXIT_SUCCESS)
}
