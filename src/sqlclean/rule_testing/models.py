"""One custom Rule test case and its result."""

from collections.abc import Mapping
from dataclasses import dataclass, field

from sqlclean.rule_testing.constants import DEFAULT_CASE_DIALECT, DEFAULT_CASE_PATH
from sqlclean.rules.models import Finding


@dataclass(frozen=True, slots=True)
class RuleFile:
    """A supporting SQL file for Rules that read more than the case's own file."""

    path: str
    source: str


@dataclass(frozen=True, slots=True)
class RuleCase:
    """SQL to check and the number of findings the Rule must report on it."""

    description: str
    source: str
    expected_finding_count: int
    path: str = DEFAULT_CASE_PATH
    dialect: str = DEFAULT_CASE_DIALECT
    files: tuple[RuleFile, ...] = ()
    options: Mapping[str, object] = field(default_factory=dict)


@dataclass(frozen=True, slots=True)
class RuleResult:
    """The Rule's findings, and the case source with every fix applied."""

    findings: tuple[Finding, ...]
    fixed_source: str

    @property
    def finding_count(self) -> int:
        """Return how many findings the Rule reported."""

        return len(self.findings)
