//! Root-relative glob patterns compiled the way every configured path list is matched.

use globset::GlobSet;

/// Compile `patterns`; `*` does not cross `/`, and `**` does.
pub fn path_patterns(patterns: &[String]) -> Result<GlobSet, String> {
    crate::discovery::_helpers::matching::glob_set(patterns)
}
