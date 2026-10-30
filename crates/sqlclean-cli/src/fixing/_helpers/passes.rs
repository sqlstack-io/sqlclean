//! The fix loop: check, apply non-overlapping fixes, keep only passes that still parse.

use std::collections::BTreeMap;

use sqlclean::sql_facts::main::file_facts::file_facts;

use crate::checking::main::check_sources::check_sources;
use crate::checking::models::{CheckOptions, CheckOutcome};
use crate::config::models::Config;
use crate::discovery::models::SourceFile;
use crate::fixing::_helpers::edit_selection::select_edits;
use crate::fixing::constants::MAX_FIX_PASSES;
use crate::fixing::models::FixOutcome;
use crate::formatting::main::format_source::format_source;
use crate::positions::main::apply_edits::apply_edits;
use crate::positions::models::Edit;

pub(crate) fn fix_sources(
    config: &Config,
    mut files: Vec<SourceFile>,
    options: &CheckOptions,
    unsafe_fixes: bool,
) -> Result<FixOutcome, String> {
    let mut applied_fixes: usize = 0;
    let mut passes: usize = 0;
    while passes < MAX_FIX_PASSES {
        let outcome: CheckOutcome = check_sources(config, &files, options)?;
        let edits: BTreeMap<String, Vec<Edit>> = select_edits(&outcome.findings, unsafe_fixes);
        let applied: usize = apply_pass(&mut files, &edits);
        if applied == 0 {
            break;
        }
        applied_fixes += applied;
        passes += 1;
    }
    let mut format_refusals: Vec<(String, String)> = Vec::new();
    for file in &mut files {
        match format_source(&file.text, &file.dialect, config.line_width) {
            Ok(formatted) => file.text = formatted,
            Err(reason) => format_refusals.push((file.path.clone(), reason)),
        }
    }
    let remaining: CheckOutcome = check_sources(config, &files, options)?;
    Ok(FixOutcome {
        files,
        applied_fixes,
        passes,
        format_refusals,
        remaining,
    })
}

/// Apply each file's edits unless they turn parsing SQL into SQL that no longer parses.
fn apply_pass(files: &mut [SourceFile], edits: &BTreeMap<String, Vec<Edit>>) -> usize {
    let mut applied: usize = 0;
    for file in files.iter_mut() {
        let Some(file_edits) = edits.get(&file.path) else {
            continue;
        };
        let fixed: String = apply_edits(&file.text, file_edits);
        if fixed == file.text {
            continue;
        }
        if let (Ok(_), Err(_)) = (
            file_facts(&file.text, &file.dialect),
            file_facts(&fixed, &file.dialect),
        ) {
            continue;
        }
        file.text = fixed;
        applied += file_edits.len();
    }
    applied
}
