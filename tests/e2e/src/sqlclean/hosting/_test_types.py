"""Test case types for the sqlclean command."""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class CheckTestCase:
    """A project, a check invocation and the findings and exit status it must give."""

    description: str
    config: str
    files: dict[str, str]
    arguments: tuple[str, ...]
    expected_exit: int
    expected_codes: tuple[str, ...]
    expected_stale: tuple[str, ...] = ()


@dataclass(frozen=True)
class RewriteTestCase:
    """A project, a fix or format invocation and the file contents it must leave."""

    description: str
    config: str
    files: dict[str, str]
    arguments: tuple[str, ...]
    expected_exit: int
    expected_files: dict[str, str] = field(default_factory=dict)


@dataclass(frozen=True)
class IdempotencyTestCase:
    """Unformatted SQL that a second format run must leave unchanged."""

    description: str
    source: str
    expected_exits: tuple[int, int]


@dataclass(frozen=True)
class CacheTestCase:
    """A project checked, rechecked without Python, then rechecked after an edit."""

    description: str
    files: dict[str, str]
    edited: dict[str, str]
    expected_exits: tuple[int, int, int]
    expected_cached_code: str
    expected_error: str


@dataclass(frozen=True)
class SummaryTestCase:
    """SQL whose text report must end with one summary line."""

    description: str
    config: str
    source: str
    expected_summary: str
