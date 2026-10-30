//! Every Rule a configuration can select, for listing and inspection.

use crate::caching::models::ResultCache;
use crate::checking::_helpers::catalogue::{builtin_rules, custom_rules, custom_sources_digest};
use crate::checking::models::RuleInfo;
use crate::config::models::Config;

/// The built-in Rules followed by the configuration's custom Rules.
pub fn rule_catalogue(config: &Config, use_cache: bool) -> Result<Vec<RuleInfo>, String> {
    let mut rules: Vec<RuleInfo> = builtin_rules();
    if !config.rules.custom.is_empty() {
        let mut cache = ResultCache::open(&config.root, use_cache);
        let digest: String = custom_sources_digest(config)?;
        rules.extend(custom_rules(config, &digest, &mut cache)?);
        cache.save(&Default::default())?;
    }
    Ok(rules)
}
