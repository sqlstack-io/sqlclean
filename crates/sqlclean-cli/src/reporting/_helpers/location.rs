//! Character offsets to line and column positions.

use std::collections::HashMap;

use crate::checking::models::Finding;
use crate::positions::models::LineIndex;
use crate::reporting::models::LocatedFinding;

pub(crate) fn locate(findings: &[Finding], texts: &HashMap<String, String>) -> Vec<LocatedFinding> {
    let mut indexes: HashMap<&str, LineIndex> = HashMap::new();
    findings
        .iter()
        .map(|finding| {
            let index: &LineIndex = indexes.entry(finding.path.as_str()).or_insert_with(|| {
                LineIndex::new(texts.get(&finding.path).map_or("", String::as_str))
            });
            let (line, column) = index.line_column(finding.start);
            let (end_line, end_column) = index.line_column(finding.end.max(finding.start));
            LocatedFinding {
                code: finding.code.clone(),
                path: finding.path.clone(),
                line,
                column,
                end_line,
                end_column,
                message: finding.message.clone(),
                remediation: finding.remediation.clone(),
                fix: finding.fix,
            }
        })
        .collect()
}
