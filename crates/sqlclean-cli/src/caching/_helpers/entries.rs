//! Read the stored cache file.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::caching::models::CacheEntry;

/// The entries stored in `file`; a missing or damaged file is an error the caller may drop.
pub(crate) fn read_entries(file: &Path) -> Result<BTreeMap<String, CacheEntry>, String> {
    let bytes: Vec<u8> = fs::read(file).map_err(|error| error.to_string())?;
    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}
