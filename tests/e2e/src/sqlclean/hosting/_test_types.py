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
    expected_unavailable: tuple[str, ...] = ()
    command: str = "check"


@dataclass(frozen=True)
class CallDepthTestCase:
    """A configuration and the findings checking 300 nested function calls in relation must give."""

    description: str
    config: str
    expected_codes: tuple[str, ...]
    expected_message: str
    relation: str = "a"
    files: dict[str, str] = field(default_factory=dict)


@dataclass(frozen=True)
class DepthAcceptedTestCase:
    """A configuration under which SQL nested up to a limit is checked and formatted."""

    description: str
    config: str
    sql: str
    expected_codes: tuple[str, ...]


@dataclass(frozen=True)
class DepthRefusedTestCase:
    """A dialect and a configuration under which SQL nested past a limit is refused, and how."""

    description: str
    config: str
    sql: str
    expected_message: str
    expected_ceiling_notes: int


@dataclass(frozen=True)
class CallDepthFormatTestCase:
    """A configuration and what formatting 300 nested function calls twice must give."""

    description: str
    config: str
    expected_exits: tuple[int, int]
    expected_start: str
    expected_message: str


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


@dataclass(frozen=True)
class SarifTestCase:
    """A project whose SARIF report must carry one result message."""

    description: str
    config: str
    source: str
    expected_message: str


@dataclass(frozen=True)
class ManyFilesTestCase:
    """Enough files for custom Rules to run on several hosts, every flagged_every-th one flagged."""

    description: str
    file_count: int
    flagged_every: int
    expected_finding_count: int


@dataclass(frozen=True)
class SkillsTestCase:
    """Files already in a project, a skills invocation and the skill entries it must leave."""

    description: str
    existing: dict[str, str]
    arguments: tuple[str, ...]
    expected_exit: int
    expected_entries: dict[str, bool]
