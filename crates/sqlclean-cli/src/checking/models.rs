//! Findings, Rule metadata and the result of one check.

use serde::{Deserialize, Serialize};

use crate::positions::models::Edit;

/// Whether a fix keeps the query's meaning (`safe`) or may change it (`unsafe`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FixSafety {
    Safe,
    Unsafe,
}

/// One Rule violation, located by character offsets in its file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub code: String,
    pub path: String,
    pub start: usize,
    pub end: usize,
    pub message: String,
    pub remediation: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edits: Vec<Edit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix: Option<FixSafety>,
}

/// The subject a Rule evaluates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleSubject {
    Statement,
    File,
    Repository,
}

/// One built-in or custom Rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleInfo {
    pub code: String,
    pub message: String,
    pub remediation: String,
    pub enabled_by_default: bool,
    pub custom: bool,
    pub subject: RuleSubject,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixable: Option<FixSafety>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}

/// A configured exception that matched no finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StaleException {
    pub rule: String,
    pub path: String,
    pub relation: Option<String>,
}

/// Every finding of one check, sorted by path, position and code.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CheckOutcome {
    pub findings: Vec<Finding>,
    pub stale_exceptions: Vec<StaleException>,
}

/// What a check evaluates beyond the configuration.
#[derive(Debug, Clone, Default)]
pub struct CheckOptions {
    /// Replaces the configured `[rules] select` when given.
    pub select: Option<Vec<String>>,
    /// Added to the configured `[rules] ignore`.
    pub ignore: Vec<String>,
    pub use_cache: bool,
}
