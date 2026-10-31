"""The custom Rule example in README.md, run as written through the Rule harness."""

from __future__ import annotations

from pathlib import Path

import pytest

from sqlclean.rules.types import RuleCheck
from sqlclean.testing import RuleCase, RuleResult, evaluate_rule
from tests.integration.src.sqlclean.rule_testing._test_types import ReadmeRuleTestCase
from tests.integration.src.sqlclean.rule_testing.helpers import readme_rule


@pytest.mark.parametrize(
    "test_case",
    [
        ReadmeRuleTestCase(
            description="a bare email column compared with a mixed-case literal",
            source="SELECT id FROM customers c WHERE c.email = 'Alice@example.com'",
            expected_texts=("c.email",),
        ),
        ReadmeRuleTestCase(
            description="LOWER on one side only still compares with a mixed-case literal",
            source="SELECT id FROM customers WHERE LOWER(email) = 'Alice@Example.com'",
            expected_texts=("email",),
        ),
        ReadmeRuleTestCase(
            description="a designated column on the right of <> in upper case",
            source="SELECT id FROM customers WHERE 'Carol@example.com' <> BILLING_EMAIL",
            expected_texts=("BILLING_EMAIL",),
        ),
        ReadmeRuleTestCase(
            description="UPPER does not normalize the other side to lower case",
            source="SELECT id FROM customers WHERE LOWER(email) = UPPER('x')",
            expected_texts=("email",),
        ),
        ReadmeRuleTestCase(
            description="a bare column against a lower-case literal is still not normalized",
            source="SELECT id FROM customers WHERE email = 'carol@example.com'",
            expected_texts=("email",),
        ),
        ReadmeRuleTestCase(
            description="a comparison inside a CTE after a multibyte literal",
            source="WITH u AS (SELECT 'é' AS m, email FROM users WHERE email = 'X@y.com') "
            "SELECT * FROM u",
            expected_texts=("email",),
        ),
        ReadmeRuleTestCase(
            description="LOWER on both sides passes",
            source="SELECT * FROM u JOIN a ON LOWER(u.email) = LOWER(a.email)",
            expected_texts=(),
        ),
        ReadmeRuleTestCase(
            description="LOWER against a lower-case literal passes",
            source="SELECT id FROM customers WHERE LOWER(email) = 'bob@example.com'",
            expected_texts=(),
        ),
        ReadmeRuleTestCase(
            description="comments, strings, other columns, LIKE and projections never match",
            source="-- email = 'A@b.c'\nSELECT email, 'email = x' AS note FROM customers "
            "WHERE email_verified = TRUE AND email LIKE '%dave%'",
            expected_texts=(),
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_readme_email_rule_when_evaluating_then_unnormalized_comparisons_are_reported(
    tmp_path: Path, test_case: ReadmeRuleTestCase
) -> None:
    rule: RuleCheck = readme_rule(tmp_path)

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
    assert result.fixed_source == test_case.source
