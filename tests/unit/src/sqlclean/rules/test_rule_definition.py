"""Tests for @rule declarations."""

from __future__ import annotations

import pytest

from sqlclean import rule
from sqlclean.rules.constants import RULE_ATTRIBUTE
from sqlclean.rules.exceptions import RuleDefinitionError
from sqlclean.rules.types import RuleCheck
from tests.unit.src.sqlclean.rules._test_types import RuleCodeTestCase
from tests.unit.src.sqlclean.rules.helpers import empty_rule


@pytest.mark.parametrize(
    "test_case",
    [
        RuleCodeTestCase(
            description="a bare custom prefix with three digits",
            code="XSQC001",
            expected_error=None,
        ),
        RuleCodeTestCase(
            description="a family of capital letters",
            code="XSQCNAME001",
            expected_error=None,
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_custom_code_when_declaring_then_the_definition_is_attached(
    test_case: RuleCodeTestCase,
) -> None:
    declared: RuleCheck = rule(code=test_case.code, message="Message", remediation="Fix it.")(
        empty_rule
    )

    assert getattr(declared, RULE_ATTRIBUTE).code == test_case.code
    assert test_case.expected_error is None


@pytest.mark.parametrize(
    "test_case",
    [
        RuleCodeTestCase(
            description="a built-in code is reserved",
            code="SQC001",
            expected_error="custom Rule code must be XSQC",
        ),
        RuleCodeTestCase(
            description="a lower-case family",
            code="XSQCname001",
            expected_error="custom Rule code must be XSQC",
        ),
        RuleCodeTestCase(
            description="two digits",
            code="XSQCNAME01",
            expected_error="custom Rule code must be XSQC",
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_invalid_code_when_declaring_then_it_is_rejected(
    test_case: RuleCodeTestCase,
) -> None:
    with pytest.raises(RuleDefinitionError) as raised:
        rule(code=test_case.code, message="Message", remediation="Fix it.")(empty_rule)

    assert str(test_case.expected_error) in str(raised.value)
