use crate::sql_lint::_helpers::engine::rule_metadata;
use crate::sql_lint::models::LintRuleMetadata;

pub fn catalogue() -> Vec<LintRuleMetadata> {
    rule_metadata()
}
