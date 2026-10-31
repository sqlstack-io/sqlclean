"""Test case types for the custom Rule harness."""

from __future__ import annotations

from dataclasses import dataclass

from sqlclean.rule_testing.models import RuleCase


@dataclass(frozen=True)
class HarnessTestCase:
    """A Rule, its case and the findings and fixed source the harness must return."""

    description: str
    rule_name: str
    rule_case: RuleCase
    expected_texts: tuple[str, ...]
    expected_fixed_source: str


@dataclass(frozen=True)
class HarnessFailureTestCase:
    """A Rule case the harness must reject, and the reason it gives."""

    description: str
    rule_name: str
    rule_case: RuleCase
    expected_error: str


@dataclass(frozen=True)
class ReadmeRuleTestCase:
    """SQL for the README custom Rule and the text of every column it must report."""

    description: str
    source: str
    expected_texts: tuple[str, ...]
