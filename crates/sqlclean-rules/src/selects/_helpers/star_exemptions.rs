//! The lone stars nobody can enumerate.

use sqlquack_syntax::expressions::queries::models::Select;
use sqlquack_syntax::{Expression, ExpressionWalk};

use crate::models::RuleInput;
use crate::selects::_helpers::stars::is_bare_star;
use crate::trees::main::principal_query::principal_query;
use crate::trees::main::unwrapped::unwrapped;

/// The SELECTs of `statement` whose projected star is exempt.
pub(crate) fn exempt_stars<'a>(
    input: &RuleInput<'a>,
    statement: &'a Expression,
) -> Vec<&'a Select> {
    let mut exempt: Vec<&Select> = Vec::new();
    let root: Option<&Expression> = principal_query(statement);
    if input.settings.allows_dynamic_output_star
        && let Some(Expression::Select(select)) = root.map(Expression::last_branch)
        && is_lone_star(select)
    {
        exempt.push(select);
    }
    let statement_ctes = statement.with_clause().or_else(|| root?.with_clause());
    for cte in statement_ctes.iter().flat_map(|with| &with.ctes) {
        if let Expression::Select(select) = unwrapped(&cte.this)
            && is_lone_star(select)
            && imports_dependency(input, select)
        {
            exempt.push(select);
        }
    }
    for node in statement.dfs() {
        for cte in node.with_clause().iter().flat_map(|with| &with.ctes) {
            if let Expression::Select(select) = unwrapped(&cte.this)
                && is_lone_star(select)
                && reads_empty_input(input, (select, &cte.alias.name))
            {
                exempt.push(select);
            }
        }
        if let Expression::Select(select) = node
            && is_lone_star(select)
            && reads_open_columns(input, select)
        {
            exempt.push(select);
        }
    }
    exempt
}

fn is_lone_star(select: &Select) -> bool {
    !select.distinct && matches!(select.expressions.as_slice(), [item] if is_bare_star(item))
}

/// The one relation a select reads, without joins.
fn lone_relation(select: &Select) -> Option<&Expression> {
    match select.from.as_ref()?.expressions.as_slice() {
        [item] if select.joins.is_empty() => Some(unwrapped(item)),
        _ => None,
    }
}

/// Whether a select only reads the one relation in FROM.
fn only_reads(select: &Select) -> bool {
    select.where_clause.is_none()
        && select.group_by.is_none()
        && select.having.is_none()
        && select.qualify.is_none()
        && select.order_by.is_none()
        && select.limit.is_none()
        && select.offset.is_none()
        && select.fetch.is_none()
}

fn imports_dependency(input: &RuleInput<'_>, select: &Select) -> bool {
    match lone_relation(select) {
        Some(Expression::Table(table)) => {
            only_reads(select)
                && table.schema.is_none()
                && table.catalog.is_none()
                && input
                    .settings
                    .dependency_identifiers
                    .contains(&table.name.name.to_ascii_lowercase())
        }
        _ => false,
    }
}

fn reads_empty_input(input: &RuleInput<'_>, (select, cte): (&Select, &str)) -> bool {
    let Some(fixture) = input.settings.fixture else {
        return false;
    };
    let (Some(empty), Some(relation)) = (fixture.empty_relation, lone_relation(select)) else {
        return false;
    };
    let read: &str = match relation {
        Expression::Table(table) if table.alias.is_none() => &table.name.name,
        Expression::Function(call) if call.args.is_empty() => &call.name,
        _ => return false,
    };
    read.eq_ignore_ascii_case(empty)
        && only_reads(select)
        && fixture
            .input_cte_prefixes
            .iter()
            .any(|prefix| cte.starts_with(prefix.as_str()))
}

/// Whether the columns are only known when run: a relation parameter's, or a PIVOT's over ANY.
fn reads_open_columns(input: &RuleInput<'_>, select: &Select) -> bool {
    let mut item: &Expression = match lone_relation(select) {
        Some(item) => item,
        None => return false,
    };
    if let Expression::Alias(alias) = item {
        item = &alias.this;
    }
    match item {
        Expression::Table(table) => is_relation_parameter(input, &table.name.name),
        Expression::Pivot(pivot) => {
            !matches!(unwrapped(&pivot.this), Expression::Pivot(_))
                && pivot
                    .fields
                    .iter()
                    .flat_map(|field| field.dfs())
                    .any(|node| matches!(node, Expression::PivotAny(_)))
        }
        _ => false,
    }
}

fn is_relation_parameter(input: &RuleInput<'_>, name: &str) -> bool {
    let lowered: String = name.to_ascii_lowercase();
    lowered
        .strip_prefix(input.settings.placeholders.relation_parameter)
        .and_then(|rest| rest.strip_suffix("__"))
        .is_some_and(|index| !index.is_empty() && index.chars().all(|digit| digit.is_ascii_digit()))
}
