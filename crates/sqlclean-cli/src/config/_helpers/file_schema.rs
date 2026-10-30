//! The configuration file as written, before defaults and validation.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::config::models::{DialectOverride, RuleException, RuleIgnore, Thresholds};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfigFile {
    pub(crate) dialect: Option<String>,
    pub(crate) include: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) exclude: Vec<String>,
    #[serde(default)]
    pub(crate) dialects: Vec<DialectOverride>,
    #[serde(default)]
    pub(crate) format: FormatTable,
    #[serde(default)]
    pub(crate) rules: RulesTable,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FormatTable {
    pub(crate) line_width: Option<usize>,
    pub(crate) indent: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RulesTable {
    pub(crate) select: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) ignore: Vec<String>,
    #[serde(default)]
    pub(crate) custom: Vec<String>,
    #[serde(default)]
    pub(crate) thresholds: Thresholds,
    #[serde(default)]
    pub(crate) rule_options: BTreeMap<String, toml::Table>,
    #[serde(default)]
    pub(crate) rule_exceptions: Vec<RuleException>,
    #[serde(default)]
    pub(crate) rule_ignores: Vec<RuleIgnore>,
}
