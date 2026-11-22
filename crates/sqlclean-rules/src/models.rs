//! What a Rule is handed, and what it returns.

use sqlquack_syntax::Expression;
use sqlquack_syntax::tokens::models::Span;

/// All a tree Rule reads: the located statements, without their SQL or tokens.
#[derive(Clone, Copy, Debug)]
pub struct RuleInput<'a> {
    pub statements: &'a [Expression],
}

/// A finding at a position of the tree, with the edit its Rule can make there.
#[derive(Debug)]
pub struct RuleFinding {
    pub span: Span,
    pub fix: Option<RuleEdit>,
}

/// An edit at a position of the tree, which the engine renders on the SQL the Rule never sees.
#[derive(Debug)]
pub struct RuleEdit {
    pub span: Span,
    pub replacement: &'static str,
    /// The edit also takes the layout after `span`, and is dropped when a comment follows that.
    pub trailing_layout: bool,
}
