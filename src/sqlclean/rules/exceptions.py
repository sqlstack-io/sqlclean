"""Errors of the custom Rule authoring API."""


class SqlCleanError(Exception):
    """Base class of every error SQLClean raises."""


class RuleDefinitionError(SqlCleanError):
    """A @rule declaration is invalid."""


class RuleOptionError(SqlCleanError):
    """A configured Rule option value is invalid."""


class RuleUsageError(SqlCleanError):
    """A Rule used the RuleContext API incorrectly."""
