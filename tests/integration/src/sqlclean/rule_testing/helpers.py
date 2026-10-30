"""The sample custom Rules, as users would write them, and their loader."""

from __future__ import annotations

import importlib.util
from importlib.machinery import ModuleSpec
from pathlib import Path
from types import ModuleType

from sqlclean.rules.types import RuleCheck

SAMPLE_RULES: str = """
from sqlclean import FixSafety, Repository, RuleContext, RuleOption, SqlFile, Statement, rule

MAX_JOINS = RuleOption.integer(
    name="max_joins", default=1, description="Most joins one statement may use.", minimum=0
)
PREFIXES = RuleOption.string_list(
    name="prefixes", default=("tmp_",), description="Forbidden table prefixes."
)


@rule(
    code="XSQCSHAPE001",
    message="Statement joins too many relations",
    remediation="Split the statement into CTEs.",
    options=(MAX_JOINS,),
)
def too_many_joins(*, statement: Statement, ctx: RuleContext) -> list:
    over_limit = statement.joins[ctx.option(MAX_JOINS):]
    return [ctx.finding(subject=statement, node=join) for join in over_limit[:1]]


@rule(
    code="XSQCNAME001",
    message="Table name uses a forbidden prefix",
    remediation="Read from the published table.",
    options=(PREFIXES,),
    enabled_by_default=True,
)
def forbidden_prefix(*, statement: Statement, ctx: RuleContext) -> list:
    prefixes = ctx.option(PREFIXES)
    return [
        ctx.finding(subject=statement, node=table)
        for table in statement.tables
        if table.name.startswith(prefixes)
    ]


@rule(
    code="XSQCSTYLE001",
    message="COUNT(1) should be COUNT(*)",
    remediation="Write COUNT(*).",
    fixable=FixSafety.SAFE,
    enabled_by_default=True,
)
def count_one(*, statement: Statement, ctx: RuleContext) -> list:
    counts = [function for function in statement.functions if function.name.upper() == "COUNT"]
    called = {function.end + 1 for function in counts}
    return [
        ctx.finding(subject=statement, node=literal, fix=[ctx.replace(node=literal, text="*")])
        for literal in statement.literals
        if literal.text == "1" and literal.start in called
    ]


@rule(
    code="XSQCFILE001",
    message="File holds more than one statement",
    remediation="Keep one statement per file.",
)
def one_statement(*, file: SqlFile, ctx: RuleContext) -> list:
    return [ctx.finding(subject=file)] if len(file.statements) > 1 else []


@rule(
    code="XSQCREPO001",
    message="Table is read by only one file",
    remediation="Inline the table or share it.",
)
def single_reader(*, repository: Repository, ctx: RuleContext) -> list:
    readers = {}
    for file in repository.files:
        for statement in file.statements:
            for table in statement.tables:
                readers.setdefault(table.name, []).append((statement, table))
    return [
        ctx.finding(subject=uses[0][0], node=uses[0][1])
        for uses in readers.values()
        if len({statement.path for statement, _ in uses}) == 1
    ]
"""


def sample_rule(directory: Path, name: str) -> RuleCheck:
    """Return the sample Rule function called name, imported from a module under directory."""

    path: Path = directory / "sample_rules.py"
    path.write_text(SAMPLE_RULES, encoding="utf-8")
    spec: ModuleSpec | None = importlib.util.spec_from_file_location("sample_rules", path)
    assert spec is not None and spec.loader is not None
    module: ModuleType = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    check: RuleCheck = getattr(module, name)
    return check
