//! One check: select Rules, run the built-ins, then custom Rules on files that parse.

use std::collections::{BTreeSet, HashSet};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::caching::main::cache_key::cache_key;
use crate::caching::models::ResultCache;
use crate::checking::_helpers::builtin::builtin_findings;
use crate::checking::_helpers::catalogue::{builtin_rules, custom_rules, custom_sources_digest};
use crate::checking::_helpers::custom::{CustomEvaluation, custom_findings};
use crate::checking::_helpers::exemptions::apply_exemptions;
use crate::checking::_helpers::selection::selected_codes;
use crate::checking::constants::{CUSTOM_RULE_PREFIX, PARSE_ERROR_CODE};
use crate::checking::models::{CheckOptions, CheckOutcome, Finding, RuleInfo};
use crate::config::models::Config;
use crate::discovery::models::SourceFile;

const BUILTIN_RESULT_KEY: &str = "builtin";

pub(crate) fn check_sources(
    config: &Config,
    files: &[SourceFile],
    options: &CheckOptions,
) -> Result<CheckOutcome, String> {
    let mut cache = ResultCache::open(&config.root, options.use_cache);
    let select: Option<&[String]> = options.select.as_deref().or(config.rules.select.as_deref());
    let builtins: Vec<RuleInfo> = builtin_rules();
    let needs_custom: bool =
        !config.rules.custom.is_empty() && select.is_none_or(selects_custom_rules);
    let digest: String = if needs_custom {
        custom_sources_digest(config)?
    } else {
        String::new()
    };
    let custom: Vec<RuleInfo> = if needs_custom {
        custom_rules(config, &digest, &mut cache)?
    } else {
        Vec::new()
    };
    validate_rule_options(config, &builtins, needs_custom.then_some(custom.as_slice()))?;
    let all: Vec<RuleInfo> = builtins.iter().chain(&custom).cloned().collect();
    let ignore: Vec<String> = config
        .rules
        .ignore
        .iter()
        .chain(&options.ignore)
        .filter(|selector| needs_custom || !selector.starts_with('X'))
        .cloned()
        .collect();
    let selected: BTreeSet<String> = selected_codes(&all, select, &ignore)?;
    let (mut findings, updated) = run_builtins(
        &BuiltinRun {
            config,
            files,
            builtins: &builtins,
            selected: &selected,
        },
        cache,
    );
    let mut cache: ResultCache = updated;
    let unparsed: HashSet<&str> = findings
        .iter()
        .filter(|finding| finding.code == PARSE_ERROR_CODE)
        .map(|finding| finding.path.as_str())
        .collect();
    let parsed: Vec<&SourceFile> = files
        .iter()
        .filter(|file| !unparsed.contains(file.path.as_str()))
        .collect();
    let custom_selected: Vec<&RuleInfo> = custom
        .iter()
        .filter(|rule| selected.contains(&rule.code))
        .collect();
    if !custom_selected.is_empty() {
        let evaluation = CustomEvaluation {
            config,
            rules: custom_selected,
            sources_digest: &digest,
        };
        findings.extend(custom_findings(&evaluation, &parsed, &mut cache)?);
    }
    let checked: HashSet<String> = files.iter().map(|file| file.path.clone()).collect();
    let (mut kept, stale_exceptions) =
        apply_exemptions(findings, &config.rules, &checked, &selected)?;
    kept.sort_by(|left, right| {
        (&left.path, left.start, left.end, &left.code).cmp(&(
            &right.path,
            right.start,
            right.end,
            &right.code,
        ))
    });
    cache
        .save(&checked)
        .map_err(|error| format!("could not write the result cache (use --no-cache): {error}"))?;
    Ok(CheckOutcome {
        findings: kept,
        stale_exceptions,
    })
}

/// Whether a selection can match custom Rules: an `XSQC` code or a prefix of `XSQC`.
fn selects_custom_rules(selectors: &[String]) -> bool {
    selectors.iter().any(|selector| {
        selector.starts_with(CUSTOM_RULE_PREFIX)
            || CUSTOM_RULE_PREFIX.starts_with(selector.as_str())
    })
}

/// The inputs of one built-in Rule run.
struct BuiltinRun<'a> {
    config: &'a Config,
    files: &'a [SourceFile],
    builtins: &'a [RuleInfo],
    selected: &'a BTreeSet<String>,
}

fn run_builtins(run: &BuiltinRun<'_>, mut cache: ResultCache) -> (Vec<Finding>, ResultCache) {
    let codes: BTreeSet<String> = run
        .builtins
        .iter()
        .filter(|rule| run.selected.contains(&rule.code))
        .map(|rule| rule.code.clone())
        .collect();
    let selection: String = codes.iter().cloned().collect::<Vec<String>>().join(",");
    let thresholds = run.config.rules.thresholds;
    let limits: String = format!(
        "{}:{}",
        thresholds.max_literal_length, thresholds.max_ranking_order_by
    );
    let keys: Vec<String> = run
        .files
        .iter()
        .map(|file| {
            cache_key(&[
                BUILTIN_RESULT_KEY,
                &selection,
                &limits,
                &file.path,
                &file.dialect,
                &file.text,
            ])
        })
        .collect();
    let cached: Vec<Option<Vec<Finding>>> = keys.iter().map(|key| cache.get(key)).collect();
    let computed: Vec<Vec<Finding>> = run
        .files
        .par_iter()
        .zip(cached)
        .map(|(file, cached)| cached.unwrap_or_else(|| builtin_findings(file, &codes, thresholds)))
        .collect();
    for ((file, key), found) in run.files.iter().zip(&keys).zip(&computed) {
        cache.put(key, Some(&file.path), found);
    }
    (computed.into_iter().flatten().collect(), cache)
}

fn validate_rule_options(
    config: &Config,
    builtins: &[RuleInfo],
    custom: Option<&[RuleInfo]>,
) -> Result<(), String> {
    for code in config.rules.rule_options.keys() {
        if builtins.iter().any(|rule| &rule.code == code) {
            return Err(format!(
                "[rules.rule_options.{code}]: built-in Rules take no options; set limits in [rules.thresholds]"
            ));
        }
        if let Some(custom) = custom
            && !custom.iter().any(|rule| &rule.code == code)
        {
            return Err(format!(
                "[rules.rule_options.{code}]: no custom Rule has this code"
            ));
        }
    }
    Ok(())
}
