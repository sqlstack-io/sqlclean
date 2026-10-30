//! Custom Rules: cached per file, Rule and options, with the host started only for misses.

use std::collections::{BTreeMap, BTreeSet};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sha2::{Digest, Sha256};
use sqlclean::sql_facts::main::file_facts::file_facts;

use crate::caching::main::cache_key::cache_key;
use crate::caching::models::ResultCache;
use crate::checking::models::{Finding, FixSafety, RuleInfo, RuleSubject};
use crate::config::models::Config;
use crate::discovery::models::SourceFile;
use crate::hosting::constants::EVALUATE_MODE;
use crate::hosting::main::run_host::run_host;
use crate::hosting::models::{HostFile, HostFinding, HostRequest, HostResponse, HostResult};

const FILE_RESULT_KEY: &str = "custom-file";
const REPOSITORY_RESULT_KEY: &str = "custom-repository";
/// The fewest files worth starting another host process for.
const MIN_FILES_PER_HOST: usize = 250;

/// What custom evaluation needs besides the files.
pub(crate) struct CustomEvaluation<'a> {
    pub(crate) config: &'a Config,
    pub(crate) rules: Vec<&'a RuleInfo>,
    pub(crate) sources_digest: &'a str,
}

/// The findings of the selected custom Rules on `files`, which must all parse.
pub(crate) fn custom_findings(
    evaluation: &CustomEvaluation<'_>,
    files: &[&SourceFile],
    cache: &mut ResultCache,
) -> Result<Vec<Finding>, String> {
    let options: BTreeMap<String, serde_json::Value> = rule_options(evaluation)?;
    let (file_rules, repository_rules): (Vec<&RuleInfo>, Vec<&RuleInfo>) = evaluation
        .rules
        .iter()
        .partition(|rule| rule.subject != RuleSubject::Repository);
    let mut findings: Vec<Finding> = Vec::new();
    let mut misses: Vec<(String, Option<String>, String)> = Vec::new();
    let mut pending: Vec<(&SourceFile, Vec<String>)> = Vec::new();
    for file in files {
        let mut missing: Vec<String> = Vec::new();
        for rule in &file_rules {
            let key: String = file_key(evaluation, rule, &options, file);
            match cache.get::<Vec<HostFinding>>(&key) {
                Some(cached) => {
                    findings.extend(cached.into_iter().map(|found| finding(rule, found)))
                }
                None => {
                    missing.push(rule.code.clone());
                    misses.push((key, Some(file.path.clone()), rule.code.clone()));
                }
            }
        }
        if !missing.is_empty() {
            pending.push((file, missing));
        }
    }
    let repository_digest: String = repository_digest(files);
    let mut missing_repository: Vec<String> = Vec::new();
    for rule in &repository_rules {
        let key: String = cache_key(&[
            REPOSITORY_RESULT_KEY,
            evaluation.sources_digest,
            &rule.code,
            &options_text(&options, &rule.code),
            &repository_digest,
        ]);
        match cache.get::<Vec<HostFinding>>(&key) {
            Some(cached) => findings.extend(cached.into_iter().map(|found| finding(rule, found))),
            None => {
                missing_repository.push(rule.code.clone());
                misses.push((key, None, rule.code.clone()));
            }
        }
    }
    if misses.is_empty() {
        return Ok(findings);
    }
    let mut host_files: Vec<HostFile> = pending
        .into_par_iter()
        .map(|(file, missing)| host_file(file, missing))
        .collect::<Result<_, _>>()?;
    if !missing_repository.is_empty() {
        host_files = with_every_file(files, host_files)?;
    }
    let results: Vec<HostResult> =
        evaluate_on_hosts(evaluation, options, host_files, missing_repository)?;
    let mut by_subject: BTreeMap<(Option<String>, String), Vec<HostFinding>> = BTreeMap::new();
    for result in results {
        by_subject.insert((result.path, result.code), result.findings);
    }
    for (key, path, code) in misses {
        let found: Vec<HostFinding> = by_subject
            .remove(&(path.clone(), code.clone()))
            .unwrap_or_default();
        cache.put(&key, path.as_deref(), &found);
        if let Some(rule) = evaluation.rules.iter().find(|rule| rule.code == code) {
            findings.extend(found.into_iter().map(|item| finding(rule, item)));
        }
    }
    Ok(findings)
}

/// Evaluate on one host per share of the files; repository Rules need every file in one host.
fn evaluate_on_hosts(
    evaluation: &CustomEvaluation<'_>,
    options: BTreeMap<String, serde_json::Value>,
    host_files: Vec<HostFile>,
    repository_rules: Vec<String>,
) -> Result<Vec<HostResult>, String> {
    let request = |files: Vec<HostFile>, repository_rules: Vec<String>| HostRequest {
        mode: EVALUATE_MODE.to_owned(),
        root: evaluation.config.root.to_string_lossy().into_owned(),
        custom: evaluation.config.rules.custom.clone(),
        options: options.clone(),
        files,
        repository_rules,
    };
    let hosts: usize =
        (host_files.len() / MIN_FILES_PER_HOST).clamp(1, rayon::current_num_threads());
    if hosts == 1 || !repository_rules.is_empty() {
        return Ok(run_host(&request(host_files, repository_rules))?.results);
    }
    let share: usize = host_files.len().div_ceil(hosts);
    let mut shares: Vec<Vec<HostFile>> = Vec::with_capacity(hosts);
    let mut rest: Vec<HostFile> = host_files;
    while rest.len() > share {
        let tail: Vec<HostFile> = rest.split_off(share);
        shares.push(rest);
        rest = tail;
    }
    shares.push(rest);
    let responses: Vec<Result<HostResponse, String>> = shares
        .into_par_iter()
        .map(|files| run_host(&request(files, Vec::new())))
        .collect();
    let mut results: Vec<HostResult> = Vec::new();
    for response in responses {
        results.extend(response?.results);
    }
    Ok(results)
}

fn rule_options(
    evaluation: &CustomEvaluation<'_>,
) -> Result<BTreeMap<String, serde_json::Value>, String> {
    let codes: BTreeSet<&str> = evaluation
        .rules
        .iter()
        .map(|rule| rule.code.as_str())
        .collect();
    let mut options: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for (code, table) in &evaluation.config.rules.rule_options {
        if codes.contains(code.as_str()) {
            let value = serde_json::to_value(table)
                .map_err(|error| format!("[rules.rule_options.{code}]: {error}"))?;
            options.insert(code.clone(), value);
        }
    }
    Ok(options)
}

fn options_text(options: &BTreeMap<String, serde_json::Value>, code: &str) -> String {
    options
        .get(code)
        .map(ToString::to_string)
        .unwrap_or_default()
}

fn file_key(
    evaluation: &CustomEvaluation<'_>,
    rule: &RuleInfo,
    options: &BTreeMap<String, serde_json::Value>,
    file: &SourceFile,
) -> String {
    cache_key(&[
        FILE_RESULT_KEY,
        evaluation.sources_digest,
        &rule.code,
        &options_text(options, &rule.code),
        &file.path,
        &file.dialect,
        &file.text,
    ])
}

fn host_file(file: &SourceFile, rules: Vec<String>) -> Result<HostFile, String> {
    Ok(HostFile {
        path: file.path.clone(),
        source: file.text.clone(),
        dialect: file.dialect.clone(),
        facts: file_facts(&file.text, &file.dialect)?,
        rules,
    })
}

/// A repository Rule reads every file, including those whose own Rules were all cached.
fn with_every_file(
    files: &[&SourceFile],
    mut host_files: Vec<HostFile>,
) -> Result<Vec<HostFile>, String> {
    for file in files {
        if !host_files.iter().any(|host| host.path == file.path) {
            host_files.push(host_file(file, Vec::new())?);
        }
    }
    host_files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(host_files)
}

fn repository_digest(files: &[&SourceFile]) -> String {
    let mut digest = Sha256::new();
    for file in files {
        for part in [&file.path, &file.dialect, &file.text] {
            digest.update(part.len().to_string().as_bytes());
            digest.update(part.as_bytes());
        }
    }
    format!("{:x}", digest.finalize())
}

fn finding(rule: &RuleInfo, found: HostFinding) -> Finding {
    let fix: Option<FixSafety> =
        (!found.edits.is_empty()).then(|| rule.fixable.unwrap_or(FixSafety::Safe));
    Finding {
        code: rule.code.clone(),
        path: found.path,
        start: found.start,
        end: found.end,
        message: found.message,
        remediation: found.remediation,
        edits: found.edits,
        fix,
    }
}
