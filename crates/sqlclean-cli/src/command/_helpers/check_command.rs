//! `sqlclean check`.

use std::collections::HashMap;
use std::io::Write;

use crate::checking::main::check_sources::check_sources;
use crate::checking::models::CheckOptions;
use crate::command::constants::{EXIT_FINDINGS, EXIT_SUCCESS};
use crate::command::models::{CheckArgs, SelectionArgs};
use crate::config::models::Config;
use crate::discovery::main::discover_files::discover_files;
use crate::reporting::main::render_findings::render_findings;

pub(crate) fn run(config: &Config, arguments: &CheckArgs) -> Result<u8, String> {
    let files = discover_files(config, &arguments.selection.paths)?;
    let outcome = check_sources(config, &files, &check_options(&arguments.selection))?;
    let texts: HashMap<String, String> = files
        .into_iter()
        .map(|file| (file.path, file.text))
        .collect();
    let report: String = render_findings(&outcome, &texts, arguments.output_format);
    let _ = std::io::stdout().write_all(report.as_bytes());
    let clean: bool = outcome.findings.is_empty() && outcome.stale_exceptions.is_empty();
    Ok(if clean { EXIT_SUCCESS } else { EXIT_FINDINGS })
}

pub(crate) fn check_options(selection: &SelectionArgs) -> CheckOptions {
    CheckOptions {
        select: selection.select.clone(),
        ignore: selection.ignore.clone(),
        use_cache: !selection.no_cache,
    }
}
