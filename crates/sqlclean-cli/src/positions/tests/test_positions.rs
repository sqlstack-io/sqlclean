use crate::positions::main::apply_edits::apply_edits;
use crate::positions::models::{Edit, LineIndex};
use crate::positions::tests::test_types::{ApplyEditsTestCase, LineColumnTestCase};

#[test]
fn given_multibyte_text_when_locating_offsets_then_lines_and_columns_count_characters() {
    let index = LineIndex::new("SELECT 'é'\nFROM t\n");
    let test_cases = [
        LineColumnTestCase {
            description: "the first character",
            offset: 0,
            expected_line_column: (1, 1),
        },
        LineColumnTestCase {
            description: "the closing quote after a two-byte character",
            offset: 9,
            expected_line_column: (1, 10),
        },
        LineColumnTestCase {
            description: "the start of the second line",
            offset: 11,
            expected_line_column: (2, 1),
        },
        LineColumnTestCase {
            description: "the end of the second line",
            offset: 16,
            expected_line_column: (2, 6),
        },
        LineColumnTestCase {
            description: "an offset past the end clamps to the last line",
            offset: 99,
            expected_line_column: (3, 1),
        },
    ];
    for test_case in &test_cases {
        assert_eq!(
            index.line_column(test_case.offset),
            test_case.expected_line_column,
            "{}",
            test_case.description
        );
    }
}

#[test]
fn given_edits_when_applying_then_offsets_count_characters() {
    let test_cases = [
        ApplyEditsTestCase {
            description: "insert, replace and delete after a multibyte character",
            text: "SELECT  'é' FROM",
            edits: &[(0, 0, "-- x\n"), (8, 11, "'e'"), (12, 16, "")],
            expected_text: "-- x\nSELECT  'e' ",
        },
        ApplyEditsTestCase {
            description: "no edits leaves the text unchanged",
            text: "SELECT 'é'",
            edits: &[],
            expected_text: "SELECT 'é'",
        },
    ];
    for test_case in &test_cases {
        let edits: Vec<Edit> = test_case
            .edits
            .iter()
            .map(|(start, end, text)| Edit {
                start: *start,
                end: *end,
                text: (*text).to_owned(),
            })
            .collect();
        assert_eq!(
            apply_edits(test_case.text, &edits),
            test_case.expected_text,
            "{}",
            test_case.description
        );
    }
}
