use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatRequest {
    pub version: u32,
    pub sql: String,
    pub dialect: String,
    #[serde(default)]
    pub max_function_call_depth: Option<usize>,
    #[serde(default)]
    pub line_width: Option<usize>,
    /// Spaces per indent step; two when absent.
    #[serde(default)]
    pub indent: Option<usize>,
    #[serde(default)]
    pub token_widths: HashMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct FormatResponse {
    pub version: u32,
    pub sql: String,
    pub changed: bool,
    pub formatted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
