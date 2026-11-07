//! The structural facts of every statement in one SQL file.

use crate::sql_facts::models::FileFacts;

/// Each statement's structural facts, or why they are unavailable for a dialect SQLQuack does not parse yet.
pub fn file_facts(sql: &str, dialect: &str) -> Result<FileFacts, String> {
    crate::sql_facts::_helpers::collection::file_facts(sql, dialect)
}
