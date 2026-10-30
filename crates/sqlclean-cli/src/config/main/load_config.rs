//! Find, read and validate the configuration for a command.

use std::path::Path;

use crate::config::models::Config;

/// The configuration at `explicit`, else the nearest `sqlclean.toml` at or above `start`.
pub fn load_config(explicit: Option<&Path>, start: &Path) -> Result<Config, String> {
    crate::config::_helpers::loading::load_config(explicit, start)
}
