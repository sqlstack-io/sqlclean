//! What provably holds at most one row.

use std::collections::{HashMap, HashSet};

use sqlquack_syntax::expressions::names::models::Literal;
use sqlquack_syntax::expressions::queries::models::{Select, Subquery};
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::{DialectFacts, RuleInput};
use crate::trees::main::is_aggregate::is_aggregate;
use crate::trees::main::is_row_number::is_row_number;

const FIRST_ROW: &str = "1";
const ROW_GENERATOR_FUNCTIONS: [&str; 16] = [
    "unnest",
    "explode",
    "explode_outer",
    "posexplode",
    "posexplode_outer",
    "inline",
    "inline_outer",
    "flatten",
    "generate_series",
    "generate_subscripts",
    "json_each",
    "json_tree",
    "json_array_elements",
    "jsonb_array_elements",
    "range",
    "stack",
];

/// The declared relations and the CTEs that hold at most one row.
pub(crate) struct OneRow {
    relations: HashSet<String>,
    ctes: HashMap<String, bool>,
    dialect: DialectFacts,
}

/// What is known of the one-row relations of the statements, each CTE judged in definition order.
pub(crate) fn one_row(input: &RuleInput<'_>) -> OneRow {
    let mut known = OneRow {
        relations: input
            .settings
            .relation_keys
            .iter()
            .filter(|(_, keys)| keys.iter().any(Vec::is_empty))
            .map(|(name, _)| name.to_ascii_lowercase())
            .collect(),
        ctes: HashMap::new(),
        dialect: input.dialect,
    };
    for node in input
        .statements
        .iter()
        .flat_map(|statement| statement.dfs())
    {
        let Some(with) = node.with_clause() else {
            continue;
        };
        for cte in &with.ctes {
            let single: bool = !with.recursive && known.at_most_one_row(&cte.this);
            let name: String = cte.alias.name.to_ascii_lowercase();
            let before: bool = known.ctes.get(&name).copied().unwrap_or(single);
            known.ctes.insert(name, before && single);
        }
    }
    known
}

impl OneRow {
    /// Whether a query expression returns at most one row.
    fn at_most_one_row(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Select(select) => self.select_one_row(select),
            Expression::Subquery(subquery) => self.subquery_one_row(subquery),
            Expression::Paren(paren) => self.at_most_one_row(&paren.this),
            Expression::Values(values) => values.expressions.len() <= 1,
            _ => false,
        }
    }

    fn subquery_one_row(&self, subquery: &Subquery) -> bool {
        !subquery.lateral
            && (subquery
                .limit
                .as_ref()
                .is_some_and(|limit| !limit.percent && at_most_one(&limit.this))
                || self.at_most_one_row(&subquery.this))
    }

    fn select_one_row(&self, select: &Select) -> bool {
        if select.limit_by.is_some() || select.connect.is_some() || !select.lateral_views.is_empty()
        {
            return false;
        }
        if limited_to_one_row(select) {
            return true;
        }
        if select.expressions.iter().any(generates_rows) {
            return false;
        }
        if select.group_by.is_none()
            && select
                .expressions
                .iter()
                .any(|expression| aggregates(expression, &self.dialect))
        {
            return true;
        }
        if select
            .qualify
            .as_ref()
            .is_some_and(|qualify| first_unpartitioned_row(&qualify.this, select))
        {
            return true;
        }
        if select.group_by.is_some() || !select.joins.is_empty() {
            return false;
        }
        match &select.from {
            None => true,
            Some(from) => match from.expressions.as_slice() {
                [relation] => self.relation_one_row(relation),
                _ => false,
            },
        }
    }

    /// Whether a relation in FROM or JOIN holds at most one row.
    pub(crate) fn relation_one_row(&self, relation: &Expression) -> bool {
        match relation {
            Expression::Table(table) => {
                if table.schema.is_some()
                    || table.catalog.is_some()
                    || table.identifier_func.is_some()
                    || table.changes.is_some()
                {
                    return false;
                }
                let name: String = table.name.name.to_ascii_lowercase();
                match self.ctes.get(&name) {
                    Some(one_row) => *one_row,
                    None => self.relations.contains(&name),
                }
            }
            Expression::Subquery(subquery) => self.subquery_one_row(subquery),
            Expression::Paren(paren) => self.relation_one_row(&paren.this),
            Expression::Values(values) => values.expressions.len() <= 1,
            _ => false,
        }
    }
}

fn limited_to_one_row(select: &Select) -> bool {
    let limited: bool = select
        .limit
        .as_ref()
        .is_some_and(|limit| !limit.percent && at_most_one(&limit.this));
    let fetched: bool = select.fetch.as_ref().is_some_and(|fetch| {
        !fetch.percent && !fetch.with_ties && fetch.count.as_ref().is_some_and(at_most_one)
    });
    let top: bool = select
        .top
        .as_ref()
        .is_some_and(|top| !top.percent && !top.with_ties && at_most_one(&top.this));
    limited || fetched || top
}

fn at_most_one(expression: &Expression) -> bool {
    match expression {
        Expression::Literal(literal) => match literal.as_ref() {
            Literal::Number(text) => text.parse::<u64>().is_ok_and(|count| count <= 1),
            _ => false,
        },
        Expression::Paren(paren) => at_most_one(&paren.this),
        _ => false,
    }
}

/// Whether a projected expression can return several rows per input row.
fn generates_rows(expression: &Expression) -> bool {
    expression.dfs().any(|node| match node {
        Expression::Unnest(_)
        | Expression::Explode(_)
        | Expression::ExplodeOuter(_)
        | Expression::GenerateSeries(_) => true,
        Expression::Function(function) => is_row_generator(&function.name),
        _ => false,
    })
}

fn is_row_generator(name: &str) -> bool {
    let name: &str = name.rsplit('.').next().unwrap_or_default();
    ROW_GENERATOR_FUNCTIONS
        .iter()
        .any(|generator| generator.eq_ignore_ascii_case(name))
}

/// Whether an expression aggregates outside any window function or subquery.
fn aggregates(expression: &Expression, dialect: &DialectFacts) -> bool {
    let mut pending: Vec<&Expression> = vec![expression];
    while let Some(node) = pending.pop() {
        if is_aggregate(node, dialect) {
            return true;
        }
        if !matches!(
            node,
            Expression::WindowFunction(_) | Expression::Subquery(_) | Expression::Select(_)
        ) {
            pending.extend(node.children());
        }
    }
    false
}

/// `QUALIFY ROW_NUMBER() OVER (ORDER BY ...) = 1` without a partition, directly or by alias.
fn first_unpartitioned_row(condition: &Expression, select: &Select) -> bool {
    let mut pending: Vec<&Expression> = vec![condition];
    while let Some(node) = pending.pop() {
        match node {
            Expression::And(operation) => pending.extend([&operation.right, &operation.left]),
            Expression::Paren(paren) => pending.push(&paren.this),
            _ if first_row_filter(node, select) => return true,
            _ => {}
        }
    }
    false
}

/// One conjunct of `first_unpartitioned_row`: an unpartitioned row number compared with 1.
fn first_row_filter(condition: &Expression, select: &Select) -> bool {
    match condition {
        Expression::Eq(operation) => {
            let compared: &Expression = match (&operation.left, &operation.right) {
                (other, Expression::Literal(literal)) | (Expression::Literal(literal), other)
                    if matches!(literal.as_ref(), Literal::Number(text) if text == FIRST_ROW) =>
                {
                    other
                }
                _ => return false,
            };
            match compared {
                Expression::WindowFunction(_) => unpartitioned_row_number(compared),
                Expression::Column(column) if column.table.is_none() => select
                    .expressions
                    .iter()
                    .any(|projection| match projection {
                        Expression::Alias(alias) => {
                            alias.alias.name.eq_ignore_ascii_case(&column.name.name)
                                && unpartitioned_row_number(&alias.this)
                        }
                        _ => false,
                    }),
                _ => false,
            }
        }
        _ => false,
    }
}

fn unpartitioned_row_number(expression: &Expression) -> bool {
    matches!(
        expression,
        Expression::WindowFunction(window)
            if is_row_number(&window.this)
                && window.over.window_name.is_none()
                && window.over.partition_by.is_empty()
    )
}
