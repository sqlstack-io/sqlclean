//! The terminal report: each finding with its source line, then a summary.

use std::collections::HashMap;
use std::fmt::Write;

use crate::checking::models::{CheckOutcome, FixSafety};
use crate::config::constants::CONFIG_FILE_NAME;
use crate::reporting::models::LocatedFinding;

pub(crate) fn render(
    outcome: &CheckOutcome,
    located: &[LocatedFinding],
    texts: &HashMap<String, String>,
) -> String {
    let mut report = String::new();
    for finding in located {
        report.push_str(&finding_text(finding, texts.get(&finding.path)));
    }
    for stale in &outcome.stale_exceptions {
        let relation: String = stale
            .relation
            .as_ref()
            .map(|relation| format!(" for relation {relation}"))
            .unwrap_or_default();
        let _ = writeln!(
            report,
            "error: stale rule exception: {} in {}{relation} matches no finding; remove it from {CONFIG_FILE_NAME}\n",
            stale.rule, stale.path
        );
    }
    let safe: usize = fix_count(located, FixSafety::Safe);
    let unsafe_only: usize = fix_count(located, FixSafety::Unsafe);
    let problems: usize = located.len() + outcome.stale_exceptions.len();
    if problems == 0 {
        report.push_str("All checks passed.\n");
    } else {
        let noun: &str = if problems == 1 { "problem" } else { "problems" };
        let _ = write!(report, "Found {problems} {noun}");
        match (safe, unsafe_only) {
            (0, 0) => {}
            (safe, 0) => {
                let _ = write!(report, " ({safe} fixable with `sqlclean fix`)");
            }
            (0, unsafe_only) => {
                let _ = write!(
                    report,
                    " ({unsafe_only} fixable with `sqlclean fix --unsafe-fixes`)"
                );
            }
            (safe, unsafe_only) => {
                let _ = write!(
                    report,
                    " ({safe} fixable with `sqlclean fix`, {unsafe_only} more with `--unsafe-fixes`)"
                );
            }
        }
        report.push_str(".\n");
    }
    report
}

fn fix_count(located: &[LocatedFinding], safety: FixSafety) -> usize {
    located
        .iter()
        .filter(|finding| finding.fix == Some(safety))
        .count()
}

fn finding_text(finding: &LocatedFinding, text: Option<&String>) -> String {
    let mut report = String::new();
    let _ = writeln!(report, "error[{}]: {}", finding.code, finding.message);
    let _ = writeln!(
        report,
        "  --> {}:{}:{}",
        finding.path, finding.line, finding.column
    );
    if let Some(line) = text.and_then(|text| text.lines().nth(finding.line - 1)) {
        let number: String = finding.line.to_string();
        let gutter: String = " ".repeat(number.len());
        let width: usize = if finding.end_line == finding.line {
            finding.end_column.saturating_sub(finding.column).max(1)
        } else {
            line.chars()
                .count()
                .saturating_sub(finding.column - 1)
                .max(1)
        };
        let _ = writeln!(report, " {gutter} |");
        let _ = writeln!(report, " {number} | {line}");
        let _ = writeln!(
            report,
            " {gutter} | {}{}",
            " ".repeat(finding.column - 1),
            "^".repeat(width)
        );
    }
    for (index, help) in finding.remediation.lines().enumerate() {
        let label: &str = if index == 0 { "  = help: " } else { "" };
        let _ = writeln!(report, "{label}{help}");
    }
    if let Some(fix) = finding.fix {
        let flag: &str = match fix {
            crate::checking::models::FixSafety::Safe => "",
            crate::checking::models::FixSafety::Unsafe => " --unsafe-fixes",
        };
        let _ = writeln!(report, "  = fix: available with `sqlclean fix{flag}`");
    }
    report.push('\n');
    report
}
