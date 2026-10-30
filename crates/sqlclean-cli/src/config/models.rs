//! The validated configuration every command reads.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;

/// The effective configuration: file values over defaults, with paths relative to `root`.
#[derive(Debug, Clone)]
pub struct Config {
    /// The directory that holds the configuration file, or the working directory without one.
    pub root: PathBuf,
    pub dialect: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub dialects: Vec<DialectOverride>,
    pub line_width: usize,
    /// Spaces per indent step in formatted SQL.
    pub indent: usize,
    pub rules: RulesConfig,
}

/// A dialect for every file whose root-relative path matches one of `paths`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialectOverride {
    pub paths: Vec<String>,
    pub dialect: String,
}

/// The `[rules]` table.
#[derive(Debug, Clone, Default)]
pub struct RulesConfig {
    /// Exact codes or family prefixes; `None` selects the default Rules.
    pub select: Option<Vec<String>>,
    pub ignore: Vec<String>,
    /// Python files or directories, relative to the root, that define custom Rules.
    pub custom: Vec<String>,
    pub thresholds: Thresholds,
    pub rule_options: BTreeMap<String, toml::Table>,
    pub rule_exceptions: Vec<RuleException>,
    pub rule_ignores: Vec<RuleIgnore>,
}

/// Project-wide limits of the SQL quality Rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Thresholds {
    pub max_literal_length: usize,
    pub max_ranking_order_by: usize,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            max_literal_length: crate::config::constants::DEFAULT_MAX_LITERAL_LENGTH,
            max_ranking_order_by: crate::config::constants::DEFAULT_MAX_RANKING_ORDER_BY,
        }
    }
}

/// One reviewed finding that is allowed: a Rule code in one file, optionally one joined relation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleException {
    pub rule: String,
    pub path: String,
    #[serde(default)]
    pub relation: Option<String>,
    #[serde(default)]
    pub cte: Option<String>,
    pub reason: String,
}

/// Rule codes or families that do not apply to files matching `paths`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleIgnore {
    pub rules: Vec<String>,
    pub paths: Vec<String>,
    pub reason: String,
}
