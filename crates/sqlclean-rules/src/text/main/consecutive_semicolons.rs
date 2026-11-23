//! `consecutive_semicolons`: SQLCLEAN015's empty statements, which the tree leaves out.

use crate::models::{Replacement, RuleEdit, RuleFinding, Terminator};

/// The terminators that only follow another, with their removal as the fix.
pub fn consecutive_semicolons(terminators: &[Terminator]) -> Vec<RuleFinding> {
    terminators
        .iter()
        .filter(|terminator| terminator.follows_terminator)
        .map(|terminator| RuleFinding {
            span: terminator.span,
            fix: Some(RuleEdit {
                span: terminator.span,
                replacement: Replacement::Text(""),
                trailing_layout: false,
            }),
        })
        .collect()
}
