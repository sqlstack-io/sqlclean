//! Locating and reading the configuration file.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::_helpers::file_schema::ConfigFile;
use crate::config::_helpers::validation::validate;
use crate::config::constants::{
    CONFIG_FILE_NAME, DEFAULT_DIALECT, DEFAULT_INCLUDE, DEFAULT_LINE_WIDTH,
};
use crate::config::models::{Config, RulesConfig};

pub(crate) fn load_config(explicit: Option<&Path>, start: &Path) -> Result<Config, String> {
    let located: Option<PathBuf> = match explicit {
        Some(path) => Some(path.to_path_buf()),
        None => find_config(start),
    };
    let (root, file): (PathBuf, ConfigFile) = match located {
        Some(path) => (
            path.parent()
                .map_or_else(|| start.to_path_buf(), Path::to_path_buf),
            read_config(&path)?,
        ),
        None => (start.to_path_buf(), ConfigFile::default()),
    };
    let config: Config = effective_config(root, file);
    validate(&config).map_err(|error| format!("{CONFIG_FILE_NAME}: {error}"))?;
    Ok(config)
}

fn find_config(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|directory| directory.join(CONFIG_FILE_NAME))
        .find(|candidate| candidate.is_file())
}

fn read_config(path: &Path) -> Result<ConfigFile, String> {
    let text: String = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    toml::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

fn effective_config(root: PathBuf, file: ConfigFile) -> Config {
    Config {
        root,
        dialect: file.dialect.unwrap_or_else(|| DEFAULT_DIALECT.to_owned()),
        include: file
            .include
            .unwrap_or_else(|| vec![DEFAULT_INCLUDE.to_owned()]),
        exclude: file.exclude,
        dialects: file.dialects,
        line_width: file.format.line_width.unwrap_or(DEFAULT_LINE_WIDTH),
        rules: RulesConfig {
            select: file.rules.select,
            ignore: file.rules.ignore,
            custom: file.rules.custom,
            thresholds: file.rules.thresholds,
            rule_options: file.rules.rule_options,
            rule_exceptions: file.rules.rule_exceptions,
            rule_ignores: file.rules.rule_ignores,
        },
    }
}
