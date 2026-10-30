//! Statement facts, located by Unicode character offsets as Python string indexes are.

use serde::Serialize;

/// Every statement of one file, in source order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FileFacts {
    pub statements: Vec<StatementFacts>,
}

/// One statement's source range, kind and the common structures it contains.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct StatementFacts {
    pub index: usize,
    pub kind: String,
    pub start: usize,
    pub end: usize,
    /// The end of the statement including its terminating semicolon, when it has one.
    pub terminated_end: usize,
    pub ctes: Vec<NamedNode>,
    pub tables: Vec<TableNode>,
    pub joins: Vec<JoinNode>,
    pub stars: Vec<StarNode>,
    pub functions: Vec<NamedNode>,
    pub columns: Vec<ColumnNode>,
    pub literals: Vec<LiteralNode>,
}

/// A named structure such as a CTE or a function call, located at its name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct NamedNode {
    pub start: usize,
    pub end: usize,
    pub name: String,
}

/// A relation read in FROM or JOIN, located at its name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TableNode {
    pub start: usize,
    pub end: usize,
    pub name: String,
    pub qualified_name: String,
    pub alias: Option<String>,
}

/// One join, located at the relation it joins.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct JoinNode {
    pub start: usize,
    pub end: usize,
    pub kind: String,
    pub relation: Option<String>,
    pub has_condition: bool,
}

/// One `*` or `qualifier.*` projection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct StarNode {
    pub start: usize,
    pub end: usize,
    pub qualifier: Option<String>,
}

/// One column reference, with its qualifier when written.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ColumnNode {
    pub start: usize,
    pub end: usize,
    pub name: String,
    pub qualifier: Option<String>,
}

/// One string or numeric literal as written.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct LiteralNode {
    pub start: usize,
    pub end: usize,
    pub kind: String,
    pub text: String,
}
