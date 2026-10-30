"""Test case types for the custom Rule authoring API."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class RuleCodeTestCase:
    """A custom Rule code @rule must accept or reject."""

    description: str
    code: str
    expected_error: str | None
