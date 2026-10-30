use serde::Serialize;

use crate::checking::models::FixSafety;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Sarif,
}

/// One finding with 1-based line and column positions, as JSON output writes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocatedFinding {
    pub code: String,
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub message: String,
    pub remediation: String,
    pub fix: Option<FixSafety>,
}
