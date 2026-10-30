//! The JSON exchanged with the Python host.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sqlclean::sql_facts::models::FileFacts;

use crate::checking::models::RuleInfo;
use crate::positions::models::Edit;

/// One request: list the custom Rules, or evaluate the named ones.
#[derive(Debug, Serialize)]
pub struct HostRequest {
    pub mode: String,
    pub root: String,
    pub custom: Vec<String>,
    pub options: BTreeMap<String, serde_json::Value>,
    pub files: Vec<HostFile>,
    pub repository_rules: Vec<String>,
}

/// One file and the statement and file Rules to evaluate on it.
#[derive(Debug, Serialize)]
pub struct HostFile {
    pub path: String,
    pub source: String,
    pub dialect: String,
    pub facts: FileFacts,
    pub rules: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct HostResponse {
    #[serde(default)]
    pub rules: Vec<RuleInfo>,
    #[serde(default)]
    pub results: Vec<HostResult>,
    #[serde(default)]
    pub error: Option<String>,
}

/// The findings of one Rule on one file, or on the repository when `path` is `None`.
#[derive(Debug, Deserialize)]
pub struct HostResult {
    pub code: String,
    pub path: Option<String>,
    pub findings: Vec<HostFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostFinding {
    pub path: String,
    pub start: usize,
    pub end: usize,
    pub message: String,
    pub remediation: String,
    #[serde(default)]
    pub edits: Vec<Edit>,
}
