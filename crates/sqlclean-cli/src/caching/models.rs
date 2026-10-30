//! The result cache of one command run.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::caching::_helpers::entries::read_entries;
use crate::caching::constants::{
    CACHE_DIRECTORY, CACHE_FILE, CACHE_PARENT_IGNORE, CACHE_PARENT_IGNORE_TEXT,
};

/// One cached value and the file it describes, `None` for repository-wide results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub path: Option<String>,
    pub value: serde_json::Value,
}

/// Cached results by key. A damaged or unreadable cache file is treated as empty.
#[derive(Debug, Default)]
pub struct ResultCache {
    file: Option<PathBuf>,
    root: PathBuf,
    entries: BTreeMap<String, CacheEntry>,
    fresh: BTreeMap<String, CacheEntry>,
}

impl ResultCache {
    /// The cache under `root`, or a cache that stores nothing when `enabled` is false.
    pub fn open(root: &Path, enabled: bool) -> Self {
        if !enabled {
            return Self::default();
        }
        let file: PathBuf = root.join(CACHE_DIRECTORY).join(CACHE_FILE);
        let entries: BTreeMap<String, CacheEntry> = read_entries(&file).unwrap_or_default();
        Self {
            file: Some(file),
            root: root.to_path_buf(),
            entries,
            fresh: BTreeMap::new(),
        }
    }

    pub fn get<T: DeserializeOwned>(&mut self, key: &str) -> Option<T> {
        self.file.as_ref()?;
        let entry: CacheEntry = self.entries.get(key)?.clone();
        match serde_json::from_value(entry.value.clone()) {
            Ok(value) => {
                self.fresh.insert(key.to_owned(), entry);
                Some(value)
            }
            Err(_) => None,
        }
    }

    pub fn put<T: Serialize>(&mut self, key: &str, path: Option<&str>, value: &T) {
        if self.file.is_none() {
            return;
        }
        let value: serde_json::Value = match serde_json::to_value(value) {
            Ok(value) => value,
            Err(_) => return,
        };
        self.fresh.insert(
            key.to_owned(),
            CacheEntry {
                path: path.map(ToOwned::to_owned),
                value,
            },
        );
    }

    /// Keep this run's entries and earlier entries for files this run did not read.
    pub fn save(self, read_paths: &HashSet<String>) -> Result<(), String> {
        let Some(file) = self.file else {
            return Ok(());
        };
        let mut kept: BTreeMap<String, CacheEntry> = self
            .entries
            .into_iter()
            .filter(|(_, entry)| {
                entry
                    .path
                    .as_ref()
                    .is_some_and(|path| !read_paths.contains(path))
            })
            .collect();
        kept.extend(self.fresh);
        let bytes: Vec<u8> = serde_json::to_vec(&kept).map_err(|error| error.to_string())?;
        let directory: &Path = file.parent().unwrap_or(&self.root);
        let failed = |error: std::io::Error| format!("{}: {error}", directory.display());
        fs::create_dir_all(directory).map_err(failed)?;
        fs::write(
            self.root.join(CACHE_PARENT_IGNORE),
            CACHE_PARENT_IGNORE_TEXT,
        )
        .map_err(failed)?;
        let staging: PathBuf = file.with_extension("tmp");
        fs::write(&staging, bytes).map_err(failed)?;
        fs::rename(&staging, &file).map_err(failed)
    }
}
