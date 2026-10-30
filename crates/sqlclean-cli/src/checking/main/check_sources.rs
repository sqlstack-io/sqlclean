//! Check SQL files against the configured Rules.

use crate::checking::models::{CheckOptions, CheckOutcome};
use crate::config::models::Config;
use crate::discovery::models::SourceFile;

/// The findings of the selected Rules on `files`, after ignores and exceptions.
pub fn check_sources(
    config: &Config,
    files: &[SourceFile],
    options: &CheckOptions,
) -> Result<CheckOutcome, String> {
    crate::checking::_helpers::orchestration::check_sources(config, files, options)
}
