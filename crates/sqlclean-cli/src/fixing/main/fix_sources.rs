//! Fix and format SQL files in memory; the caller decides whether to write them.

use crate::checking::models::CheckOptions;
use crate::config::models::Config;
use crate::discovery::models::SourceFile;
use crate::fixing::models::FixOutcome;

/// Apply safe fixes (and unsafe ones when `unsafe_fixes`) until stable, then format.
pub fn fix_sources(
    config: &Config,
    files: Vec<SourceFile>,
    options: &CheckOptions,
    unsafe_fixes: bool,
) -> Result<FixOutcome, String> {
    crate::fixing::_helpers::passes::fix_sources(config, files, options, unsafe_fixes)
}
