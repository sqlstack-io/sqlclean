//! Render one check's outcome in the requested format.

use std::collections::HashMap;

use crate::checking::models::CheckOutcome;
use crate::reporting::models::OutputFormat;

/// The report for `outcome`; `texts` holds each checked file's text by path.
pub fn render_findings(
    outcome: &CheckOutcome,
    texts: &HashMap<String, String>,
    format: OutputFormat,
) -> String {
    let located = crate::reporting::_helpers::location::locate(&outcome.findings, texts);
    match format {
        OutputFormat::Text => crate::reporting::_helpers::text::render(outcome, &located, texts),
        OutputFormat::Json => crate::reporting::_helpers::machine::render_json(outcome, &located),
        OutputFormat::Sarif => crate::reporting::_helpers::machine::render_sarif(&located),
    }
}
