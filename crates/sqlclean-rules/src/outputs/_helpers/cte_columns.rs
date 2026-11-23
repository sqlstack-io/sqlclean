//! SQLCLEAN042: output columns of non-final CTEs that no later step reads.

use std::collections::{HashMap, HashSet};

use sqlquack_syntax::expressions::queries::models::{Cte, Join, JoinKind, Select, With};
use sqlquack_syntax::tokens::models::Span;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{OutputFinding, OutputKind, Replacement, RuleEdit, RuleFinding, RuleInput};
use crate::outputs::_helpers::removal_safety::{
    expression_refusal, reads_whole_rows, select_refusal,
};
use crate::queries::main::clauses_around::clauses_around;
use crate::queries::main::columns::columns;
use crate::queries::main::lower::lower;
use crate::queries::main::projections::projections;
use crate::queries::main::qualifier::qualifier;
use crate::queries::main::query_within::query_within;
use crate::queries::main::sources::sources;
use crate::queries::models::Projection;
use crate::trees::main::is_generated::is_generated;
use crate::trees::main::join_chains::join_chains;
use crate::trees::main::walk_in_scope::walk_in_scope;

const HARNESS_PREFIX: &str = "__";
const COLUMNS_FUNCTION: &str = "COLUMNS";

/// What one later step reads from a CTE.
#[derive(Default)]
struct Usage {
    names: HashSet<String>,
    everything: bool,
    passes_to: Vec<String>,
    read: bool,
}

impl Usage {
    fn merged(mut self, other: Self) -> Self {
        self.names.extend(other.names);
        self.everything |= other.everything;
        self.passes_to.extend(other.passes_to);
        self.read |= other.read;
        self
    }
}

/// One CTE whose unused outputs are being reported.
struct CteOutputs<'a> {
    cte: &'a Cte,
    usage: Option<&'a Usage>,
}

/// The unread outputs of the non-final CTEs of `statements`' queries.
pub(crate) fn unused_columns(input: &RuleInput<'_>) -> Vec<OutputFinding> {
    let mut found: Vec<OutputFinding> = Vec::new();
    for written in input.statements {
        let statement: &Expression = query_within(written);
        if !matches!(
            statement,
            Expression::Select(_)
                | Expression::Union(_)
                | Expression::Intersect(_)
                | Expression::Except(_)
        ) {
            continue;
        }
        let Some(with) = statement.with_clause().filter(|with| !with.recursive) else {
            continue;
        };
        let names: Vec<String> = with.ctes.iter().map(|cte| lower(&cte.alias.name)).collect();
        let mut terminal: Vec<&Expression> = terminal_nodes(statement, with);
        terminal.extend(
            clauses_around(written)
                .into_iter()
                .flat_map(|clause| clause.dfs()),
        );
        let used: HashMap<String, Usage> = reads(input, (&with.ctes, &names), terminal);
        let earlier = with.ctes.len().saturating_sub(1);
        for (cte, name) in with.ctes.iter().zip(&names).take(earlier) {
            if name.starts_with(HARNESS_PREFIX)
                || input.settings.externally_referenced_ctes.contains(name)
            {
                continue;
            }
            found.extend(unused_outputs(
                input,
                CteOutputs {
                    cte,
                    usage: used.get(name),
                },
            ));
        }
    }
    found
}

/// Names each CTE's later readers use, following `SELECT *` pass-through to the reader.
fn reads<'a>(
    input: &RuleInput<'_>,
    (ctes, names): (&'a [Cte], &[String]),
    terminal: Vec<&'a Expression>,
) -> HashMap<String, Usage> {
    let bodies: Vec<(Option<String>, Vec<&Expression>)> = ctes
        .iter()
        .map(|cte| (Some(lower(&cte.alias.name)), cte.this.dfs().collect()))
        .chain(std::iter::once((None, terminal)))
        .collect();
    let known: HashSet<&str> = names.iter().map(String::as_str).collect();
    let outputs: Vec<Option<HashSet<String>>> = ctes.iter().map(output_names).collect();
    let mut readers: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, (_, body)) in bodies.iter().enumerate() {
        for name in read_relations(body) {
            if known.contains(name.as_str()) {
                readers.entry(name).or_default().push(index);
            }
        }
    }
    let mut usage: HashMap<String, Usage> = HashMap::new();
    for (index, name) in names.iter().enumerate() {
        let mut current: Usage = Usage::default();
        if index + 1 == names.len() {
            current.everything = true;
        }
        for &reader in readers.get(name.as_str()).into_iter().flatten() {
            if reader > index {
                let (reader_name, body) = &bodies[reader];
                let target: Target<'_> = Target {
                    input,
                    name,
                    outputs: outputs[index].as_ref(),
                };
                current = current.merged(reader_usage(&target, reader_name.clone(), body));
            }
        }
        current.names.extend(self_reads(&ctes[index].this));
        usage.insert(name.clone(), current);
    }
    for index in (0..names.len()).rev() {
        let passes: Vec<String> = usage[&names[index]].passes_to.clone();
        for reader in passes {
            let (everything, inherited): (bool, Vec<String>) =
                usage.get(&reader).map_or((true, Vec::new()), |next| {
                    (next.everything, next.names.iter().cloned().collect())
                });
            if let Some(current) = usage.get_mut(&names[index]) {
                current.everything |= everything;
                current.names.extend(inherited);
            }
        }
    }
    usage
}

/// The output names of a CTE, or None when a star or unnamed projection hides them.
fn output_names(cte: &Cte) -> Option<HashSet<String>> {
    let Expression::Select(select) = query_within(&cte.this) else {
        return None;
    };
    projections(select)
        .into_iter()
        .map(|item| match item {
            Projection::Named { name, .. } => Some(name),
            _ => None,
        })
        .collect()
}

/// The statement's query without its WITH clause, depth first and borrowed, not recursively cloned.
fn terminal_nodes<'a>(statement: &'a Expression, with: &'a With) -> Vec<&'a Expression> {
    let skipped: Vec<&Expression> = with
        .ctes
        .iter()
        .map(|cte| &cte.this)
        .chain(with.search.as_deref())
        .collect();
    let mut nodes: Vec<&Expression> = vec![statement];
    for child in statement.children() {
        if !skipped.iter().any(|body| std::ptr::eq(*body, child)) {
            nodes.extend(child.dfs());
        }
    }
    nodes
}

fn read_relations(body: &[&Expression]) -> HashSet<String> {
    body.iter()
        .filter_map(|node| match node {
            Expression::Table(table) => Some(lower(&table.name.name)),
            _ => None,
        })
        .collect()
}

/// The CTE whose readers are being inspected.
struct Target<'a> {
    input: &'a RuleInput<'a>,
    name: &'a str,
    outputs: Option<&'a HashSet<String>>,
}

/// How one reader, given as its nodes depth first from its root, uses the target CTE.
fn reader_usage(target: &Target<'_>, reader: Option<String>, body: &[&Expression]) -> Usage {
    let name: &str = target.name;
    let mut usage: Usage = Usage::default();
    let mut aliases: HashSet<String> = HashSet::new();
    let mut top_level_star: bool = false;
    for &node in body {
        let Expression::Select(select) = node else {
            continue;
        };
        let local: Vec<String> = sources(select)
            .into_iter()
            .filter(|(_, relation, _)| relation.as_deref() == Some(name))
            .map(|(alias, _, _)| alias)
            .collect();
        if local.is_empty() {
            continue;
        }
        let joins: Vec<&Join> = join_chains(select)
            .into_iter()
            .flat_map(|(_, joins)| joins)
            .collect();
        if joins.iter().any(|join| {
            matches!(
                join.kind,
                JoinKind::Natural
                    | JoinKind::NaturalLeft
                    | JoinKind::NaturalRight
                    | JoinKind::NaturalFull
            )
        }) {
            usage.everything = true;
        }
        for join in joins {
            usage
                .names
                .extend(join.using.iter().map(|identifier| lower(&identifier.name)));
        }
        let top: bool = body.first().is_some_and(|root| std::ptr::eq(node, *root));
        if reads_as_row(select, &local, target.outputs) {
            usage.everything = true;
        }
        for projection in projections(select) {
            if let Projection::Star(star) = projection
                && star
                    .table
                    .as_ref()
                    .is_none_or(|table| local.contains(&lower(&table.name)))
            {
                if top
                    && reader.is_some()
                    && star.except.is_none()
                    && star.replace.is_none()
                    && !reads_whole_rows(select)
                {
                    top_level_star = true;
                } else {
                    usage.everything = true;
                }
            }
        }
        aliases.extend(local);
    }
    if aliases.is_empty() {
        return usage;
    }
    usage.read = true;
    usage.everything |= body.iter().any(
        |node| matches!(node, Expression::Column(column) if is_generated(target.input, &column.name.name)),
    );
    if top_level_star && let Some(reader) = reader {
        usage.passes_to.push(reader);
    }
    for column in body.iter().filter_map(|node| match node {
        Expression::Column(column) => Some(column),
        _ => None,
    }) {
        if qualifier(column).is_none_or(|table| aliases.contains(&table)) {
            usage.names.insert(lower(&column.name.name));
        }
    }
    usage
}

/// Whether a reader consumes whole rows: a nested star, `COLUMNS(...)`, or the alias itself.
fn reads_as_row(select: &Select, local: &[String], outputs: Option<&HashSet<String>>) -> bool {
    for clause in clause_roots(select) {
        for node in walk_in_scope(clause) {
            if reads_row_at(select, node, local, outputs) {
                return true;
            }
        }
    }
    false
}

fn reads_row_at(
    select: &Select,
    node: &Expression,
    local: &[String],
    outputs: Option<&HashSet<String>>,
) -> bool {
    match node {
        Expression::Star(star) => {
            let projected: bool = select
                .expressions
                .iter()
                .any(|item| std::ptr::eq(item, node));
            let table: Option<String> = star.table.as_ref().map(|table| lower(&table.name));
            !projected && table.is_none_or(|table| local.contains(&table))
        }
        Expression::BracedWildcard(_) => true,
        Expression::Function(function) => function.name.eq_ignore_ascii_case(COLUMNS_FUNCTION),
        Expression::Column(column) => {
            let read: String = lower(&column.name.name);
            let named_output: bool = outputs.is_some_and(|names| names.contains(&read));
            column.table.is_none() && local.contains(&read) && !named_output
        }
        _ => false,
    }
}

/// Root expressions of every clause of one select, without its FROM relations.
fn clause_roots(select: &Select) -> Vec<&Expression> {
    let mut roots: Vec<&Expression> = select.expressions.iter().collect();
    roots.extend(select.where_clause.iter().map(|clause| &clause.this));
    roots.extend(select.having.iter().map(|clause| &clause.this));
    roots.extend(select.qualify.iter().map(|clause| &clause.this));
    if let Some(group) = &select.group_by {
        roots.extend(group.expressions.iter());
    }
    if let Some(order) = &select.order_by {
        roots.extend(order.expressions.iter().map(|ordered| &ordered.this));
    }
    roots.extend(
        join_chains(select)
            .into_iter()
            .flat_map(|(_, joins)| joins)
            .filter_map(|join| join.on.as_ref()),
    );
    roots
}

/// Lateral alias reuse: a later projection or a clause of the CTE reading an earlier output.
fn self_reads(body: &Expression) -> HashSet<String> {
    let mut reads: HashSet<String> = HashSet::new();
    let Expression::Select(select) = query_within(body) else {
        return reads;
    };
    let mut earlier: HashSet<String> = HashSet::new();
    for item in projections(select) {
        if let Projection::Named { name, expression } = item {
            for read in unqualified_reads(expression) {
                if earlier.contains(&read) && read != name {
                    reads.insert(read);
                }
            }
            earlier.insert(name);
        }
    }
    let mut clauses: Vec<&Expression> = Vec::new();
    clauses.extend(select.where_clause.iter().map(|clause| &clause.this));
    clauses.extend(
        select
            .group_by
            .iter()
            .flat_map(|group| group.expressions.iter()),
    );
    clauses.extend(select.having.iter().map(|clause| &clause.this));
    clauses.extend(select.qualify.iter().map(|clause| &clause.this));
    if let Some(order) = &select.order_by {
        clauses.extend(order.expressions.iter().map(|ordered| &ordered.this));
    }
    clauses.extend(clauses_around(body));
    for clause in clauses {
        reads.extend(
            unqualified_reads(clause)
                .into_iter()
                .filter(|read| earlier.contains(read)),
        );
    }
    reads
}

fn unqualified_reads(expression: &Expression) -> Vec<String> {
    columns(expression)
        .into_iter()
        .filter(|column| qualifier(column).is_none())
        .map(|column| lower(&column.name.name))
        .collect()
}

fn unused_outputs(input: &RuleInput<'_>, outputs: CteOutputs<'_>) -> Vec<OutputFinding> {
    let CteOutputs { cte, usage } = outputs;
    let Expression::Select(select) = query_within(&cte.this) else {
        return Vec::new();
    };
    let Some(usage) = usage.filter(|usage| !usage.everything && usage.read) else {
        return Vec::new();
    };
    if !cte.columns.is_empty() {
        return Vec::new();
    }
    let output: Vec<Projection<'_>> = projections(select);
    let written: Vec<Span> = select
        .item_spans
        .0
        .iter()
        .filter_map(|item| item.operation())
        .collect();
    if written.len() != output.len()
        || !output
            .iter()
            .all(|item| matches!(item, Projection::Named { .. }))
    {
        return Vec::new();
    }
    let grouped: HashSet<String> = select
        .group_by
        .iter()
        .flat_map(|group| group.expressions.iter())
        .flat_map(columns)
        .map(|column| lower(&column.name.name))
        .collect();
    let select_refused: Option<&'static str> = select_refusal(select, output.len(), &input.dialect);
    let mut found: Vec<OutputFinding> = Vec::new();
    for (index, item) in output.iter().enumerate() {
        let Projection::Named { name, expression } = item else {
            continue;
        };
        if usage.names.contains(name) || is_generated(input, name) {
            continue;
        }
        let grouping_key: bool = matches!(expression, Expression::Column(source) if grouped.contains(&lower(&source.name.name)));
        let kind: OutputKind = if select.distinct {
            OutputKind::Distinct
        } else if select
            .group_by
            .as_ref()
            .is_some_and(|group| group.all == Some(true))
        {
            OutputKind::GroupByAll
        } else if grouping_key {
            OutputKind::GroupKey
        } else {
            OutputKind::Plain
        };
        let refusal: Option<&'static str> =
            select_refused.or_else(|| expression_refusal(expression));
        let fix: Option<RuleEdit> = match refusal {
            Some(_) => None,
            None => removal(&written, index),
        };
        found.push(OutputFinding {
            finding: RuleFinding {
                span: written[index],
                fix,
            },
            kind,
            refusal,
        });
    }
    found
}

/// The comma-inclusive removal of item `index`.
fn removal(written: &[Span], index: usize) -> Option<RuleEdit> {
    let item: Span = *written.get(index)?;
    let span: Span = match (written.get(index + 1), index.checked_sub(1)) {
        (Some(next), _) => Span {
            end: next.start,
            ..item
        },
        (None, Some(before)) => Span {
            start: written[before].end,
            ..item
        },
        (None, None) => return None,
    };
    Some(RuleEdit {
        span,
        replacement: Replacement::Text(""),
        trailing_layout: false,
    })
}
