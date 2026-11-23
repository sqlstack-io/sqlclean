//! The ranking windows of SELECTs, each with the order it resolves to.

use std::collections::{HashMap, HashSet};

use sqlquack_syntax::expressions::functions::models::WindowFunction;
use sqlquack_syntax::expressions::queries::models::{NamedWindow, Over, Select};
use sqlquack_syntax::tokens::models::Span;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::queries::main::lower::lower;
use crate::trees::main::call_name::call_name;
use crate::trees::main::call_positions::call_positions;
use crate::trees::main::walk_in_scope::walk_in_scope;

/// One ranking call's window.
pub(crate) struct RankingWindow<'a> {
    pub(crate) function: &'static str,
    pub(crate) called: &'a Expression,
    pub(crate) window: &'a WindowFunction,
    pub(crate) select: &'a Select,
    /// The WINDOW clauses its OVER can name: the SELECT's own, then those after enclosing parentheses.
    pub(crate) windows: Vec<&'a NamedWindow>,
}

pub(crate) fn ranking_windows(statements: &[Expression]) -> Vec<RankingWindow<'_>> {
    let mut found: Vec<RankingWindow<'_>> = Vec::new();
    for statement in statements {
        let outer: HashMap<*const Select, Vec<&NamedWindow>> = enclosing_windows(statement);
        for node in statement.dfs() {
            let Expression::Select(select) = node else {
                continue;
            };
            let mut windows: Vec<&NamedWindow> = select.windows.iter().flatten().collect();
            if let Some(enclosing) = outer.get(&std::ptr::from_ref(select.as_ref())) {
                windows.extend(enclosing.iter().rev());
            }
            for local in walk_in_scope(node) {
                let Expression::WindowFunction(window) = local else {
                    continue;
                };
                let called: &Expression = &window.this;
                let function: &'static str = match called {
                    Expression::RowNumber(_) => "ROW_NUMBER",
                    Expression::Rank(_) => "RANK",
                    Expression::DenseRank(_) => "DENSE_RANK",
                    Expression::FirstValue(_) => "FIRST_VALUE",
                    Expression::LastValue(_) => "LAST_VALUE",
                    written => match call_name(written).map(str::to_ascii_uppercase).as_deref() {
                        Some("ROW_NUMBER") => "ROW_NUMBER",
                        Some("RANK") => "RANK",
                        Some("DENSE_RANK") => "DENSE_RANK",
                        Some("FIRST_VALUE") => "FIRST_VALUE",
                        Some("LAST_VALUE") => "LAST_VALUE",
                        _ => continue,
                    },
                };
                found.push(RankingWindow {
                    function,
                    called,
                    window,
                    select,
                    windows: windows.clone(),
                });
            }
        }
    }
    found
}

/// Each SELECT's enclosing WINDOW clauses, from parentheses and outer SELECTs, outermost first.
fn enclosing_windows(statement: &Expression) -> HashMap<*const Select, Vec<&NamedWindow>> {
    let mut found: HashMap<*const Select, Vec<&NamedWindow>> = HashMap::new();
    for node in statement.dfs() {
        let (windows, inside): (&[NamedWindow], Box<dyn Iterator<Item = &Expression>>) = match node
        {
            Expression::Subquery(subquery) => match &subquery.windows {
                Some(windows) => (windows, Box::new(subquery.this.dfs())),
                None => continue,
            },
            Expression::Select(select) => match &select.windows {
                Some(windows) => (windows, Box::new(node.dfs().skip(1))),
                None => continue,
            },
            _ => continue,
        };
        for inner in inside {
            if let Expression::Select(select) = inner {
                found
                    .entry(std::ptr::from_ref(select.as_ref()))
                    .or_default()
                    .extend(windows);
            }
        }
    }
    found
}

pub(crate) fn effective_over<'a>(over: &'a Over, windows: &[&'a NamedWindow]) -> Option<&'a Over> {
    let mut current: &Over = over;
    let mut seen: HashSet<String> = HashSet::new();
    loop {
        if !current.order_by.is_empty() || current.window_name.is_none() {
            return Some(current);
        }
        let name: String = lower(&current.window_name.as_ref()?.name);
        if !seen.insert(name.clone()) {
            return None;
        }
        current = &windows
            .iter()
            .find(|window| lower(&window.name.name) == name)?
            .spec;
    }
}

/// Where the ranking call's name is written; a typed call keeps none, so its whole window stands for it.
pub(crate) fn named_at(ranking: &RankingWindow<'_>) -> Option<Span> {
    call_positions(ranking.called).0.0.or(ranking.window.span.0)
}
