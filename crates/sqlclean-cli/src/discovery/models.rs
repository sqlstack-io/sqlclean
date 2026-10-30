use std::path::PathBuf;

/// One SQL file a command reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// The path relative to the configuration root, with `/` separators.
    pub path: String,
    pub absolute: PathBuf,
    pub text: String,
    pub dialect: String,
}
