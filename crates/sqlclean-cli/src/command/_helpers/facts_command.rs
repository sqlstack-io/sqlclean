//! `sqlclean facts`: the statement facts of SQL on standard input, for Rule test harnesses.

use std::io::{Read, Write};

use sqlclean::sql_facts::main::file_facts::file_facts;

use crate::command::constants::EXIT_SUCCESS;
use crate::command::models::FactsArgs;

pub(crate) fn run(arguments: &FactsArgs) -> Result<u8, String> {
    let mut sql = String::new();
    std::io::stdin()
        .read_to_string(&mut sql)
        .map_err(|error| error.to_string())?;
    let facts = file_facts(&sql, &arguments.dialect)?;
    let json: String = serde_json::to_string(&facts).map_err(|error| error.to_string())?;
    let _ = writeln!(std::io::stdout(), "{json}");
    Ok(EXIT_SUCCESS)
}
