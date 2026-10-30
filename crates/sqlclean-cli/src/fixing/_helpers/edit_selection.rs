//! Choose the fixes one pass can apply together.

use std::collections::BTreeMap;

use crate::checking::models::{Finding, FixSafety};
use crate::positions::models::Edit;

/// Edits by path from every eligible finding whose edits overlap no earlier accepted edit.
pub(crate) fn select_edits(
    findings: &[Finding],
    unsafe_fixes: bool,
) -> BTreeMap<String, Vec<Edit>> {
    let mut eligible: Vec<&Finding> = findings
        .iter()
        .filter(|finding| is_eligible(finding, unsafe_fixes))
        .collect();
    eligible.sort_by_key(|finding| (finding.path.clone(), finding.start, finding.code.clone()));
    let mut accepted: BTreeMap<String, Vec<Edit>> = BTreeMap::new();
    for finding in eligible {
        let edits: &mut Vec<Edit> = accepted.entry(finding.path.clone()).or_default();
        if is_compatible(&finding.edits, edits) {
            edits.extend(finding.edits.iter().cloned());
        }
    }
    for edits in accepted.values_mut() {
        edits.sort_by_key(|edit| (edit.start, edit.end));
    }
    accepted.retain(|_, edits| !edits.is_empty());
    accepted
}

fn is_eligible(finding: &Finding, unsafe_fixes: bool) -> bool {
    match finding.fix {
        Some(FixSafety::Safe) => true,
        Some(FixSafety::Unsafe) => unsafe_fixes,
        None => false,
    }
}

/// The finding's edits overlap neither each other nor any accepted edit.
fn is_compatible(edits: &[Edit], accepted: &[Edit]) -> bool {
    edits.iter().enumerate().all(|(index, edit)| {
        !overlaps_any(edit, accepted) && !overlaps_any(edit, &edits[index + 1..])
    })
}

fn overlaps_any(edit: &Edit, others: &[Edit]) -> bool {
    others.iter().any(|other| overlaps(edit, other))
}

/// Two edits touch the same characters, or both insert at the same point.
fn overlaps(left: &Edit, right: &Edit) -> bool {
    let both_insert_here: bool =
        left.start == left.end && right.start == right.end && left.start == right.start;
    both_insert_here || (left.start < right.end && right.start < left.end)
}
