"""Errors the custom Rule host reports to the sqlclean command."""

from sqlclean.rules.exceptions import SqlCleanError


class RuleLoadingError(SqlCleanError):
    """A custom Rule module could not be imported, or two Rules share a code."""


class RuleEvaluationError(SqlCleanError):
    """A custom Rule raised an exception or returned something other than findings."""
