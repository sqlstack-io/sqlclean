//! Read every SQL file the given paths select.

use std::path::PathBuf;

use crate::config::models::Config;
use crate::discovery::models::SourceFile;

/// The configured SQL files under `paths` (the configuration root when empty), sorted by path.
pub fn discover_files(config: &Config, paths: &[PathBuf]) -> Result<Vec<SourceFile>, String> {
    crate::discovery::_helpers::walking::discover_files(config, paths)
}
