"""A Rule body for declaration tests."""

from __future__ import annotations

from sqlclean import RuleContext, Statement
from sqlclean.rules.models import Finding


def empty_rule(*, statement: Statement, ctx: RuleContext) -> list[Finding]:
    """Report nothing."""

    return []
