//! Every finding over the lint corpus is one that was accepted: a change, a SQLQuack bump's included, is accepted explicitly.

use crate::lint::tests::helpers::{change_report, corpus_changes};
use crate::lint::tests::test_types::LintCorpusTestCase;

#[test]
fn given_the_lint_corpus_when_linting_every_dialect_then_each_finding_is_the_accepted_one()
-> Result<(), String> {
    let test_cases = [LintCorpusTestCase {
        description: "the CI lint corpus on all six dialects",
        statements: "src/lint/tests/fixtures/lint_corpus.jsonl",
        accepted: "src/lint/tests/fixtures/lint_corpus_accepted.tsv",
        expected_changes: &[],
    }];

    for test_case in test_cases {
        let changes: Vec<String> = corpus_changes(test_case.statements, test_case.accepted)?;
        assert_eq!(
            changes,
            test_case.expected_changes,
            "{}: {}",
            test_case.description,
            change_report(&changes, test_case.accepted)
        );
    }
    Ok(())
}
