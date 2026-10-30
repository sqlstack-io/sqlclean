//! `sqlclean format`.

use std::io::Write;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::command::_helpers::diffs::{print_diff, write_file};
use crate::command::constants::{EXIT_FINDINGS, EXIT_SUCCESS};
use crate::command::models::FormatArgs;
use crate::config::models::Config;
use crate::discovery::main::discover_files::discover_files;
use crate::formatting::main::format_source::format_source;

pub(crate) fn run(config: &Config, arguments: &FormatArgs) -> Result<u8, String> {
    let files = discover_files(config, &arguments.paths)?;
    let mut stderr = std::io::stderr();
    let (mut changed, mut refused): (usize, usize) = (0, 0);
    let results: Vec<Result<String, String>> = files
        .par_iter()
        .map(|file| format_source(&file.text, &file.dialect, config.line_width, config.indent))
        .collect();
    for (file, result) in files.iter().zip(results) {
        let formatted: String = match result {
            Ok(formatted) => formatted,
            Err(reason) => {
                refused += 1;
                let _ = writeln!(stderr, "{}: could not format: {reason}", file.path);
                continue;
            }
        };
        if formatted == file.text {
            continue;
        }
        changed += 1;
        if arguments.diff {
            print_diff(file, &formatted);
        } else if arguments.check {
            let _ = writeln!(std::io::stdout(), "Would reformat: {}", file.path);
        } else {
            write_file(file, &formatted)?;
        }
    }
    let unchanged: usize = files.len() - changed - refused;
    let verb: &str = if arguments.check || arguments.diff {
        "would be reformatted"
    } else {
        "reformatted"
    };
    let _ = writeln!(
        stderr,
        "{changed} file(s) {verb}, {unchanged} file(s) already formatted."
    );
    let failed: bool = refused > 0 || ((arguments.check || arguments.diff) && changed > 0);
    Ok(if failed { EXIT_FINDINGS } else { EXIT_SUCCESS })
}
