"""Failures the custom Rule test harness reports."""

from sqlclean.rules.exceptions import SqlCleanError


class RuleCaseError(SqlCleanError):
    """A Rule case did not produce what it expects, or the Rule is not deterministic."""
