"""The introductory custom Rule examples, run as written through the Rule harness."""

from __future__ import annotations

from pathlib import Path

import pytest

from sqlclean.rules.types import RuleCheck
from sqlclean.testing import RuleCase, RuleResult, evaluate_rule
from tests.integration.src.sqlclean.rule_testing._test_types import ReadmeRuleTestCase
from tests.integration.src.sqlclean.rule_testing.helpers import document as document
from tests.integration.src.sqlclean.rule_testing.helpers import readme_rule


@pytest.mark.parametrize(
    "test_case",
    [
        ReadmeRuleTestCase(
            description="the documented SQL reports the function name",
            source="SELECT IFNULL(discount, 0) FROM orders;",
            expected_texts=("IFNULL",),
        ),
        ReadmeRuleTestCase(
            description="mixed-case calls nested in a CTE and subquery are reported",
            source="WITH totals AS (SELECT ifnull(discount, 0) AS discount FROM orders) "
            "SELECT (SELECT IfNull(MAX(discount), 0) FROM totals)",
            expected_texts=("ifnull", "IfNull"),
        ),
        ReadmeRuleTestCase(
            description="a later statement after a multibyte literal uses file character offsets",
            source="SELECT 'é';\nSELECT IFNULL(discount, 0) FROM orders;",
            expected_texts=("IFNULL",),
        ),
        ReadmeRuleTestCase(
            description="the recommended COALESCE replacement passes",
            source="SELECT COALESCE(discount, 0) FROM orders;",
            expected_texts=(),
        ),
        ReadmeRuleTestCase(
            description="comments, strings, identifiers and longer function names never match",
            source="-- IFNULL(discount, 0)\n"
            "SELECT 'IFNULL(discount, 0)' AS note, \"IFNULL\", "
            "my_ifnull(discount, 0) FROM orders /* IFNULL(discount, 0) */",
            expected_texts=(),
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_introductory_rule_when_evaluating_then_only_ifnull_calls_are_reported(
    tmp_path: Path, document: Path, test_case: ReadmeRuleTestCase
) -> None:
    rule: RuleCheck = readme_rule(tmp_path, document)

    result: RuleResult = evaluate_rule(
        rule=rule,
        test_case=RuleCase(
            description=test_case.description,
            source=test_case.source,
            expected_finding_count=len(test_case.expected_texts),
            dialect="duckdb",
        ),
    )
    texts: tuple[str, ...] = tuple(
        test_case.source[finding.start : finding.end] for finding in result.findings
    )

    assert texts == test_case.expected_texts
    assert all(finding.edits == () for finding in result.findings)
    assert result.fixed_source == test_case.source
