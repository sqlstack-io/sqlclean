//! Every finding over the lint corpus is one that was accepted: a change, a SQLQuack bump's included, is accepted explicitly.

use crate::lint::tests::corpus::{change_report, corpus_changes};

#[test]
fn given_the_lint_corpus_when_linting_every_dialect_then_each_finding_is_the_accepted_one()
-> Result<(), String> {
    let changes: Vec<String> = corpus_changes()?;
    assert!(changes.is_empty(), "{}", change_report(&changes));
    Ok(())
}
