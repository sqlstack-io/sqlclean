//! JSON and SARIF reports.

use serde_json::{Value, json};

use crate::checking::models::CheckOutcome;
use crate::reporting::models::LocatedFinding;

const SARIF_SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";
const SARIF_VERSION: &str = "2.1.0";
const TOOL_NAME: &str = "sqlclean";
const TOOL_URI: &str = "https://github.com/sqlstack-io/sqlclean";

pub(crate) fn render_json(outcome: &CheckOutcome, located: &[LocatedFinding]) -> String {
    let report: Value = json!({
        "findings": located,
        "stale_exceptions": outcome.stale_exceptions,
        "summary": {
            "findings": located.len(),
            "fixable": located.iter().filter(|finding| finding.fix.is_some()).count(),
            "stale_exceptions": outcome.stale_exceptions.len(),
        },
    });
    format!("{report:#}\n")
}

pub(crate) fn render_sarif(located: &[LocatedFinding]) -> String {
    let mut codes: Vec<&str> = located
        .iter()
        .map(|finding| finding.code.as_str())
        .collect();
    codes.sort_unstable();
    codes.dedup();
    let results: Vec<Value> = located
        .iter()
        .map(|finding| {
            json!({
                "ruleId": finding.code,
                "level": "error",
                "message": { "text": format!("{}. {}", finding.message.trim_end_matches('.'), finding.remediation) },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": finding.path },
                        "region": {
                            "startLine": finding.line,
                            "startColumn": finding.column,
                            "endLine": finding.end_line,
                            "endColumn": finding.end_column.max(finding.column + 1),
                        },
                    },
                }],
            })
        })
        .collect();
    let report: Value = json!({
        "$schema": SARIF_SCHEMA,
        "version": SARIF_VERSION,
        "runs": [{
            "tool": { "driver": {
                "name": TOOL_NAME,
                "informationUri": TOOL_URI,
                "version": env!("CARGO_PKG_VERSION"),
                "rules": codes.iter().map(|code| json!({ "id": code })).collect::<Vec<Value>>(),
            }},
            "results": results,
        }],
    });
    format!("{report:#}\n")
}
