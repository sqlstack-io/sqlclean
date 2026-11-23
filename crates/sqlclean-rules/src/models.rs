//! What a Rule is handed, and what it returns.

use std::collections::{HashMap, HashSet};

use sqlquack_syntax::Expression;
use sqlquack_syntax::expressions::names::models::Identifier;
use sqlquack_syntax::tokens::models::Span;

/// All a tree Rule reads: the located statements, without their SQL or tokens.
#[derive(Clone, Copy, Debug)]
pub struct RuleInput<'a> {
    pub statements: &'a [Expression],
    pub dialect: DialectFacts,
    pub settings: RuleSettings<'a>,
}

/// What a Rule needs to know of the dialect.
#[derive(Clone, Copy, Debug)]
pub struct DialectFacts {
    /// `UNION DISTINCT` is valid, so a bare UNION may be spelled out.
    pub union_distinct: bool,
    /// The resolved name of a name as written, quoted or not.
    pub resolved_name: fn(&str, bool) -> String,
    /// Whether two resolved names name the same object.
    pub same_name: fn(&str, &str) -> bool,
    /// The SQL targets no particular engine, so other dialects' aggregate names still count.
    pub generic: bool,
    /// The parser keeps aggregate calls as plain calls, so they are told apart by name.
    pub plain_call_aggregates: bool,
    /// Other dialects' aggregates the dialect has none of, so a call of one has an unknown kind.
    pub foreign_aggregates: &'static [&'static str],
    /// The dialect's own aggregates, for a parser that keeps them as plain calls.
    pub named_aggregates: &'static [&'static str],
    /// Of those, the aggregates that are scalar with more than one argument.
    pub single_argument_aggregates: &'static [&'static str],
    /// JSON parse functions that take one string argument, in upper case.
    pub json_parse_functions: &'static [&'static str],
    /// The parser writes `IS DISTINCT FROM` as NOT over `IS NOT DISTINCT FROM`.
    pub negated_distinct_from: bool,
}

/// What the request tells the Rules, with names in lower case.
#[derive(Clone, Copy, Debug)]
pub struct RuleSettings<'a> {
    pub external_identifiers: &'a HashSet<String>,
    pub dependency_identifiers: &'a HashSet<String>,
    pub externally_referenced_ctes: &'a HashSet<String>,
    pub relation_keys: &'a HashMap<String, Vec<Vec<String>>>,
    pub macro_outputs: &'a [MacroOutput<'a>],
    /// The generated wrapper around a test or macro body; absent for an ordinary query.
    pub fixture: Option<Fixture<'a>>,
    pub allows_dynamic_output_star: bool,
    pub max_ranking_order_by: usize,
    pub placeholders: Placeholders,
}

/// The character range one macro call's output fills, and the relations its arguments pass.
#[derive(Clone, Copy, Debug)]
pub struct MacroOutput<'a> {
    pub start: usize,
    pub end: usize,
    pub relations: &'a [String],
}

/// The names a fixture harness generates.
#[derive(Clone, Copy, Debug)]
pub struct Fixture<'a> {
    pub cte_prefixes: &'a [String],
    pub input_cte_prefixes: &'a [String],
    pub empty_relation: Option<&'a str>,
}

/// The identifier prefixes a host writes in place of template code.
#[derive(Clone, Copy, Debug)]
pub struct Placeholders {
    pub generated: &'static [&'static str],
    pub macro_parameter: &'static str,
    /// The prefix of a relation parameter, written `<prefix><index>__`.
    pub relation_parameter: &'static str,
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
    pub replacement: Replacement,
    /// The edit also takes the layout after `span`, and is dropped when a comment follows that.
    pub trailing_layout: bool,
}

/// What an edit writes.
#[derive(Clone, Copy, Debug)]
pub enum Replacement {
    Text(&'static str),
    /// A word written between two tokens, set off by a space on each side the source leaves bare.
    Spaced(&'static str),
    /// The source at `kept`, without the layout around it, between two fixed texts.
    Wrapped {
        before: &'static str,
        kept: Span,
        after: &'static str,
    },
}

/// A cross join that can multiply rows, with the relation and CTE its message names.
#[derive(Debug)]
pub struct CrossJoinFinding<'a> {
    pub span: Span,
    pub relation: Option<JoinedName<'a>>,
    pub cte: Option<&'a str>,
}

/// How a cross join's relation is named.
#[derive(Debug)]
pub enum JoinedName<'a> {
    /// A name to print as the parser read it.
    Read(&'a str),
    /// An unqualified table, which may be a sentinel standing for another name.
    Table(&'a Identifier),
    /// A qualified table's parts in order, an omitted schema being none.
    Qualified(Vec<Option<&'a Identifier>>),
    /// An alias, to print as authored.
    Alias(&'a Identifier),
}

/// A JSON payload column parsed more than once in one SELECT.
#[derive(Debug)]
pub struct JsonParseFinding<'a> {
    pub span: Span,
    pub function: &'static str,
    pub qualifier: Option<&'a Identifier>,
    pub column: &'a Identifier,
    pub count: usize,
}

/// A ranking window that sorts by too many keys, or whose order is not proven unique.
#[derive(Debug)]
pub struct RankingFinding {
    pub span: Span,
    pub sort_keys: usize,
    pub exceeds_cap: bool,
    /// ROW_NUMBER has no ORDER BY at all.
    pub unordered: bool,
    pub unproven: bool,
}

/// A CTE output column that no later step reads.
#[derive(Debug)]
pub struct OutputFinding {
    pub finding: RuleFinding,
    pub kind: OutputKind,
    /// Why the column is not removed automatically, when it is not.
    pub refusal: Option<&'static str>,
}

/// What the unused column is to its CTE, which decides the advice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputKind {
    Plain,
    Distinct,
    GroupByAll,
    GroupKey,
}

/// A statement terminator, as the terminators-only view of the text has it.
#[derive(Clone, Copy, Debug)]
pub struct Terminator {
    pub span: Span,
    /// Nothing but layout and comments is between this terminator and the one before it.
    pub follows_terminator: bool,
}

/// A string literal, as the literals-only view of the text has it: its range and length, not its text.
#[derive(Clone, Copy, Debug)]
pub struct StringLiteral {
    pub span: Span,
    pub length: usize,
}

/// Comments beside one token, as the comments-only view of the text has them.
#[derive(Clone, Copy, Debug)]
pub struct CommentPlacement {
    /// What the comments are attached to, whose text and kind the view leaves out.
    pub anchor: Span,
    /// The comments follow the anchor on its line.
    pub trailing: bool,
    pub block: bool,
    /// A blank line separates the comments from the anchor.
    pub detached: bool,
    /// The anchor is a comma, a closing parenthesis or a terminator.
    pub before_punctuation: bool,
}
