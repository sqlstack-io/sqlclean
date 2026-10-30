//! `sqlclean fix`.

use std::collections::HashMap;
use std::io::Write;

use crate::command::_helpers::check_command::check_options;
use crate::command::_helpers::diffs::{print_diff, write_file};
use crate::command::constants::{EXIT_FINDINGS, EXIT_SUCCESS};
use crate::command::models::FixArgs;
use crate::config::models::Config;
use crate::discovery::main::discover_files::discover_files;
use crate::fixing::main::fix_sources::fix_sources;
use crate::reporting::main::render_findings::render_findings;
use crate::reporting::models::OutputFormat;

pub(crate) fn run(config: &Config, arguments: &FixArgs) -> Result<u8, String> {
    let originals = discover_files(config, &arguments.selection.paths)?;
    let outcome = fix_sources(
        config,
        originals.clone(),
        &check_options(&arguments.selection),
        arguments.unsafe_fixes,
    )?;
    let mut changed_files: usize = 0;
    for (original, fixed) in originals.iter().zip(&outcome.files) {
        if original.text == fixed.text {
            continue;
        }
        changed_files += 1;
        if arguments.diff {
            print_diff(original, &fixed.text);
        } else {
            write_file(original, &fixed.text)?;
        }
    }
    let mut stderr = std::io::stderr();
    for (path, reason) in &outcome.format_refusals {
        let _ = writeln!(stderr, "{path}: could not format: {reason}");
    }
    let summary: &str = if arguments.diff {
        "Would fix {fixes} problem(s) and change {files} file(s)."
    } else {
        "Fixed {fixes} problem(s) and changed {files} file(s)."
    };
    let _ = writeln!(
        stderr,
        "{}",
        summary
            .replace("{fixes}", &outcome.applied_fixes.to_string())
            .replace("{files}", &changed_files.to_string())
    );
    if !arguments.diff {
        let texts: HashMap<String, String> = outcome
            .files
            .iter()
            .map(|file| (file.path.clone(), file.text.clone()))
            .collect();
        let report: String = render_findings(&outcome.remaining, &texts, OutputFormat::Text);
        let _ = std::io::stdout().write_all(report.as_bytes());
    }
    let remaining: bool = !outcome.remaining.findings.is_empty()
        || !outcome.remaining.stale_exceptions.is_empty()
        || !outcome.format_refusals.is_empty()
        || (arguments.diff && changed_files > 0);
    Ok(if remaining {
        EXIT_FINDINGS
    } else {
        EXIT_SUCCESS
    })
}
