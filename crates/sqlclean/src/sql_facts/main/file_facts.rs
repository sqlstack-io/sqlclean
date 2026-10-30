//! The structural facts of every statement in one SQL file.

use crate::sql_facts::models::FileFacts;

/// Parse `sql` under `dialect` and return each statement's structural facts.
pub fn file_facts(sql: &str, dialect: &str) -> Result<FileFacts, String> {
    crate::sql_facts::_helpers::collection::file_facts(sql, dialect)
}
