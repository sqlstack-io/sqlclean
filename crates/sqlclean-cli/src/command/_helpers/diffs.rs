//! Unified diffs and file writes for commands that change files.

use std::fs;
use std::io::Write;

use similar::TextDiff;

use crate::discovery::models::SourceFile;

const CONTEXT_LINES: usize = 3;

pub(crate) fn print_diff(file: &SourceFile, changed: &str) {
    let diff = TextDiff::from_lines(file.text.as_str(), changed);
    let rendered: String = diff
        .unified_diff()
        .context_radius(CONTEXT_LINES)
        .header(&format!("a/{}", file.path), &format!("b/{}", file.path))
        .to_string();
    let _ = std::io::stdout().write_all(rendered.as_bytes());
}

pub(crate) fn write_file(file: &SourceFile, changed: &str) -> Result<(), String> {
    fs::write(&file.absolute, changed)
        .map_err(|error| format!("could not write {}: {error}", file.path))
}
