use crate::checking::models::{Finding, FixSafety};
use crate::fixing::_helpers::edit_selection::select_edits;
use crate::fixing::tests::helpers::finding;
use crate::fixing::tests::test_types::EditSelectionTestCase;

#[test]
fn given_overlapping_and_unsafe_fixes_when_selecting_then_keeps_first_eligible_non_overlapping() {
    let findings: Vec<Finding> = vec![
        finding("A", Some(FixSafety::Safe), &[(0, 5, "x")]),
        finding("B", Some(FixSafety::Safe), &[(3, 8, "y")]),
        finding("C", Some(FixSafety::Safe), &[(5, 5, "z"), (9, 10, "")]),
        finding("D", Some(FixSafety::Unsafe), &[(20, 21, "u")]),
        finding("E", None, &[(30, 31, "n")]),
        finding("F", Some(FixSafety::Safe), &[(9, 9, "w")]),
        finding("G", Some(FixSafety::Safe), &[(9, 9, "v")]),
    ];
    let test_cases = [
        EditSelectionTestCase {
            description: "safe only: overlaps, a second insert at one point and unfixable findings are skipped",
            unsafe_fixes: false,
            expected_spans: &[(0, 5), (5, 5), (9, 9), (9, 10)],
        },
        EditSelectionTestCase {
            description: "unsafe fixes are added when allowed",
            unsafe_fixes: true,
            expected_spans: &[(0, 5), (5, 5), (9, 9), (9, 10), (20, 21)],
        },
    ];
    for test_case in &test_cases {
        let selected = select_edits(&findings, test_case.unsafe_fixes);
        let spans: Vec<(usize, usize)> = selected
            .get("q.sql")
            .map(|edits| edits.iter().map(|edit| (edit.start, edit.end)).collect())
            .unwrap_or_default();
        assert_eq!(spans, test_case.expected_spans, "{}", test_case.description);
    }
}
