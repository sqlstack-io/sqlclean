//! Resolve `select` and `ignore` selectors, exact codes or family prefixes, to Rule codes.

use std::collections::BTreeSet;

use crate::checking::models::RuleInfo;

pub(crate) fn selected_codes(
    rules: &[RuleInfo],
    select: Option<&[String]>,
    ignore: &[String],
) -> Result<BTreeSet<String>, String> {
    let mut selected: BTreeSet<String> = match select {
        None => rules
            .iter()
            .filter(|rule| rule.enabled_by_default)
            .map(|rule| rule.code.clone())
            .collect(),
        Some(selectors) => expand(rules, selectors)?,
    };
    for code in expand(rules, ignore)? {
        selected.remove(&code);
    }
    Ok(selected)
}

fn expand(rules: &[RuleInfo], selectors: &[String]) -> Result<BTreeSet<String>, String> {
    let mut codes: BTreeSet<String> = BTreeSet::new();
    for selector in selectors {
        let matches: Vec<&RuleInfo> = rules
            .iter()
            .filter(|rule| rule.code.starts_with(selector.as_str()))
            .collect();
        if matches.is_empty() {
            return Err(format!("Rule selector '{selector}' matches no Rule"));
        }
        codes.extend(matches.into_iter().map(|rule| rule.code.clone()));
    }
    Ok(codes)
}
