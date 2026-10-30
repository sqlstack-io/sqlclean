//! The Rules a configuration can select: the built-ins and its custom Rules.

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use sqlclean::sql_lint::main::catalogue::catalogue;
use sqlclean::sql_lint::main::default_rules::default_rules;

use crate::caching::main::cache_key::cache_key;
use crate::caching::models::ResultCache;
use crate::checking::_helpers::builtin::builtin_fix_safety;
use crate::checking::constants::FIXABLE_BUILTIN_RULES;
use crate::checking::models::{RuleInfo, RuleSubject};
use crate::config::models::Config;
use crate::hosting::constants::CATALOGUE_MODE;
use crate::hosting::main::run_host::run_host;
use crate::hosting::models::HostRequest;

const CATALOGUE_KEY: &str = "custom-catalogue";
const PYTHON_SUFFIX: &str = "py";
const PYTHON_CACHE_DIRECTORY: &str = "__pycache__";

pub(crate) fn builtin_rules() -> Vec<RuleInfo> {
    let defaults: Vec<&str> = default_rules();
    catalogue()
        .into_iter()
        .map(|rule| RuleInfo {
            code: rule.code.to_owned(),
            message: rule.message.to_owned(),
            remediation: rule.remediation.to_owned(),
            enabled_by_default: defaults.contains(&rule.code),
            custom: false,
            subject: RuleSubject::Statement,
            fixable: FIXABLE_BUILTIN_RULES
                .contains(&rule.code)
                .then(|| builtin_fix_safety(rule.code)),
            module: None,
        })
        .collect()
}

/// A digest of every Python file under the configured custom Rule paths.
pub(crate) fn custom_sources_digest(config: &Config) -> Result<String, String> {
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in &config.rules.custom {
        let path: PathBuf = config.root.join(entry);
        if !path.exists() {
            return Err(format!("[rules] custom path does not exist: {entry}"));
        }
        files.extend(python_files(&path)?);
    }
    files.sort();
    let mut digest = Sha256::new();
    for file in files {
        let text: Vec<u8> =
            fs::read(&file).map_err(|error| format!("{}: {error}", file.display()))?;
        digest.update(file.to_string_lossy().as_bytes());
        digest.update(text.len().to_string().as_bytes());
        digest.update(&text);
    }
    Ok(format!("{:x}", digest.finalize()))
}

/// The custom Rules, from the cache while their sources are unchanged, else from the host.
pub(crate) fn custom_rules(
    config: &Config,
    digest: &str,
    cache: &mut ResultCache,
) -> Result<Vec<RuleInfo>, String> {
    let key: String = cache_key(&[CATALOGUE_KEY, digest, &config.rules.custom.join("\n")]);
    if let Some(rules) = cache.get::<Vec<RuleInfo>>(&key) {
        return Ok(rules);
    }
    let response = run_host(&HostRequest {
        mode: CATALOGUE_MODE.to_owned(),
        root: config.root.to_string_lossy().into_owned(),
        custom: config.rules.custom.clone(),
        options: Default::default(),
        files: Vec::new(),
        repository_rules: Vec::new(),
    })?;
    cache.put(&key, None, &response.rules);
    Ok(response.rules)
}

/// The Python files at `path`, or under it when it is a directory.
fn python_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        let is_python: bool = path
            .extension()
            .is_some_and(|suffix| suffix == PYTHON_SUFFIX);
        return Ok(if is_python {
            vec![path.to_path_buf()]
        } else {
            Vec::new()
        });
    }
    let mut files: Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(path).map_err(|error| format!("{}: {error}", path.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with('.') && name != PYTHON_CACHE_DIRECTORY {
            files.extend(python_files(&entry.path())?);
        }
    }
    Ok(files)
}
