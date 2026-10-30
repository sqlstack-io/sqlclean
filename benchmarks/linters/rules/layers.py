"""Benchmark custom Rules: layering and determinism checks a data team might write."""

from sqlclean import Finding, RuleContext, Statement, rule

NONDETERMINISTIC_FUNCTIONS: frozenset[str] = frozenset(
    {"CURRENT_DATE", "CURRENT_TIMESTAMP", "GETDATE", "NOW", "RANDOM", "SYSDATE", "UUID_STRING"}
)


@rule(
    code="XSQCLAYER001",
    message="Mart reads a raw source",
    remediation="Read the staging model instead.",
    enabled_by_default=True,
)
def marts_read_staging(*, statement: Statement, ctx: RuleContext) -> list[Finding]:
    """Flag every raw table a mart reads."""

    if not statement.path.startswith("marts/"):
        return []
    return [
        ctx.finding(subject=statement, node=table)
        for table in statement.tables
        if table.qualified_name.startswith("raw.")
    ]


@rule(
    code="XSQCLAYER002",
    message="Staging model reads another model",
    remediation="Staging models read only raw sources and seeds.",
    enabled_by_default=True,
)
def staging_reads_sources(*, statement: Statement, ctx: RuleContext) -> list[Finding]:
    """Flag every table a staging model reads that is neither raw, a seed nor one of its CTEs."""

    if not statement.path.startswith("staging/"):
        return []
    ctes: set[str] = {cte.name.lower() for cte in statement.ctes}
    return [
        ctx.finding(subject=statement, node=table)
        for table in statement.tables
        if not table.qualified_name.startswith(("raw.", "seeds."))
        and table.name.lower() not in ctes
    ]


@rule(
    code="XSQCDET001",
    message="Model calls a nondeterministic function",
    remediation="Pass the time or seed in as a column so reruns give the same rows.",
    enabled_by_default=True,
)
def deterministic_functions(*, statement: Statement, ctx: RuleContext) -> list[Finding]:
    """Flag calls such as CURRENT_TIMESTAMP that change between runs."""

    return [
        ctx.finding(subject=statement, node=function)
        for function in statement.functions
        if function.name.upper() in NONDETERMINISTIC_FUNCTIONS
    ]
