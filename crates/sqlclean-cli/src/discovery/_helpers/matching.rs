//! Root-relative path matching for include, exclude and per-path dialect patterns.

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::config::models::Config;

/// Compiled path patterns of one configuration.
pub(crate) struct PathMatchers {
    include: GlobSet,
    exclude: GlobSet,
    dialects: Vec<(GlobSet, String)>,
    default_dialect: String,
}

impl PathMatchers {
    pub(crate) fn new(config: &Config) -> Result<Self, String> {
        let mut dialects: Vec<(GlobSet, String)> = Vec::new();
        for entry in &config.dialects {
            dialects.push((glob_set(&entry.paths)?, entry.dialect.clone()));
        }
        Ok(Self {
            include: glob_set(&config.include)?,
            exclude: glob_set(&config.exclude)?,
            dialects,
            default_dialect: config.dialect.clone(),
        })
    }

    pub(crate) fn is_included(&self, path: &str) -> bool {
        self.include.is_match(path) && !self.is_excluded(path)
    }

    pub(crate) fn is_excluded(&self, path: &str) -> bool {
        self.exclude.is_match(path)
    }

    /// The dialect of the first matching per-path entry, or the configured default.
    pub(crate) fn dialect(&self, path: &str) -> String {
        self.dialects
            .iter()
            .find(|(patterns, _)| patterns.is_match(path))
            .map_or_else(
                || self.default_dialect.clone(),
                |(_, dialect)| dialect.clone(),
            )
    }
}

/// Compile root-relative glob patterns; `*` does not cross `/`, and `**` does.
pub(crate) fn glob_set(patterns: &[String]) -> Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob: Glob = globset::GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .map_err(|error| format!("invalid path pattern '{pattern}': {error}"))?;
        builder.add(glob);
    }
    builder.build().map_err(|error| error.to_string())
}
