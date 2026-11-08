use std::path::PathBuf;

/// One SQL file a command reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// The path relative to the configuration root, with `/` separators.
    pub path: String,
    pub absolute: PathBuf,
    /// The SQL, without the byte order mark when the file starts with one.
    pub text: String,
    /// The file starts with a UTF-8 byte order mark, which a write keeps.
    pub bom: bool,
    pub dialect: String,
}
