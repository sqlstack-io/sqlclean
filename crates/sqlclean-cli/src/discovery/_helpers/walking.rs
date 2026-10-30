//! Walk the requested paths, honouring ignore files, and read the selected SQL files.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use crate::config::models::Config;
use crate::discovery::_helpers::matching::PathMatchers;
use crate::discovery::models::SourceFile;

pub(crate) fn discover_files(
    config: &Config,
    paths: &[PathBuf],
) -> Result<Vec<SourceFile>, String> {
    let matchers = PathMatchers::new(config)?;
    let roots: Vec<PathBuf> = if paths.is_empty() {
        vec![config.root.clone()]
    } else {
        paths.to_vec()
    };
    let mut selected: BTreeMap<String, PathBuf> = BTreeMap::new();
    for root in &roots {
        if !root.exists() {
            return Err(format!("{}: no such file or directory", root.display()));
        }
        if root.is_file() {
            let relative: String = relative_path(&config.root, root);
            if !matchers.is_excluded(&relative) {
                selected.insert(relative, root.clone());
            }
            continue;
        }
        for entry in WalkBuilder::new(root).build() {
            let entry = entry.map_err(|error| error.to_string())?;
            if !entry.file_type().is_some_and(|kind| kind.is_file()) {
                continue;
            }
            let relative: String = relative_path(&config.root, entry.path());
            if matchers.is_included(&relative) {
                selected.insert(relative, entry.path().to_path_buf());
            }
        }
    }
    selected
        .into_iter()
        .map(|(path, absolute)| read_source(&matchers, path, absolute))
        .collect()
}

fn read_source(
    matchers: &PathMatchers,
    path: String,
    absolute: PathBuf,
) -> Result<SourceFile, String> {
    let text: String =
        fs::read_to_string(&absolute).map_err(|error| format!("could not read {path}: {error}"))?;
    Ok(SourceFile {
        dialect: matchers.dialect(&path),
        path,
        absolute,
        text,
    })
}

/// `path` relative to `root` with `/` separators, or as given when it lies outside the root.
pub(crate) fn relative_path(root: &Path, path: &Path) -> String {
    let absolute_root: PathBuf = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let absolute_path: PathBuf = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let relative: &Path = absolute_path.strip_prefix(&absolute_root).unwrap_or(path);
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<String>>()
        .join("/")
}
