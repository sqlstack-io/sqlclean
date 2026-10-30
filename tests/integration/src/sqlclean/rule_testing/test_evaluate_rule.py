"""Tests for evaluate_rule over SQL parsed by the native sqlclean executable."""

from __future__ import annotations

from pathlib import Path

import pytest

from sqlclean.rules.exceptions import SqlCleanError
from sqlclean.rules.types import RuleCheck
from sqlclean.testing import RuleCase, RuleFile, RuleResult, evaluate_rule
from tests.integration.src.sqlclean.rule_testing._test_types import (
    HarnessFailureTestCase,
    HarnessTestCase,
)
from tests.integration.src.sqlclean.rule_testing.helpers import sample_rule

JOINED: str = "SELECT a.id FROM a JOIN b ON a.id = b.id JOIN c ON b.id = c.id"


@pytest.mark.parametrize(
    "test_case",
    [
        HarnessTestCase(
            description="the default limit reports the first join over it",
            rule_name="too_many_joins",
            rule_case=RuleCase(description="two joins", source=JOINED, expected_finding_count=1),
            expected_texts=("JOIN c ON b.id = c.id",),
            expected_fixed_source=JOINED,
        ),
        HarnessTestCase(
            description="a configured limit replaces the default",
            rule_name="too_many_joins",
            rule_case=RuleCase(
                description="limit two",
                source=JOINED,
                expected_finding_count=0,
                options={"max_joins": 2},
            ),
            expected_texts=(),
            expected_fixed_source=JOINED,
        ),
        HarnessTestCase(
            description="a fix after a multibyte literal edits by character offset",
            rule_name="count_one",
            rule_case=RuleCase(
                description="count one",
                source="SELECT 'é' AS mark, COUNT(1) AS n FROM t",
                expected_finding_count=1,
            ),
            expected_texts=("1",),
            expected_fixed_source="SELECT 'é' AS mark, COUNT(*) AS n FROM t",
        ),
        HarnessTestCase(
            description="a file Rule sees every statement of the file",
            rule_name="one_statement",
            rule_case=RuleCase(
                description="two statements",
                source="SELECT 1;\nSELECT 2;\n",
                expected_finding_count=1,
            ),
            expected_texts=("",),
            expected_fixed_source="SELECT 1;\nSELECT 2;\n",
        ),
        HarnessTestCase(
            description="a repository Rule reads the supporting files",
            rule_name="single_reader",
            rule_case=RuleCase(
                description="orders read twice, customers once",
                source="SELECT * FROM orders",
                path="models/a.sql",
                files=(RuleFile(path="models/b.sql", source="SELECT * FROM orders, customers"),),
                expected_finding_count=1,
            ),
            expected_texts=("customers",),
            expected_fixed_source="SELECT * FROM orders",
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_rule_case_when_evaluating_then_findings_and_fixed_source_match(
    tmp_path: Path, test_case: HarnessTestCase
) -> None:
    rule: RuleCheck = sample_rule(tmp_path, test_case.rule_name)

    result: RuleResult = evaluate_rule(rule=rule, test_case=test_case.rule_case)
    sources: dict[str, str] = {
        test_case.rule_case.path: test_case.rule_case.source,
        **{file.path: file.source for file in test_case.rule_case.files},
    }
    texts: tuple[str, ...] = tuple(
        sources[finding.path][finding.start : finding.end] for finding in result.findings
    )

    assert texts == test_case.expected_texts
    assert result.fixed_source == test_case.expected_fixed_source


@pytest.mark.parametrize(
    "test_case",
    [
        HarnessFailureTestCase(
            description="a wrong expected count fails with both counts",
            rule_name="too_many_joins",
            rule_case=RuleCase(description="two joins", source=JOINED, expected_finding_count=0),
            expected_error="expected 0 findings, got 1",
        ),
        HarnessFailureTestCase(
            description="an option below its minimum is rejected",
            rule_name="too_many_joins",
            rule_case=RuleCase(
                description="negative limit",
                source=JOINED,
                expected_finding_count=0,
                options={"max_joins": -1},
            ),
            expected_error="max_joins must be at least 0",
        ),
        HarnessFailureTestCase(
            description="an undeclared option is rejected",
            rule_name="too_many_joins",
            rule_case=RuleCase(
                description="unknown option",
                source=JOINED,
                expected_finding_count=0,
                options={"limit": 2},
            ),
            expected_error="has no option limit",
        ),
        HarnessFailureTestCase(
            description="SQL the parser rejects fails the case",
            rule_name="too_many_joins",
            rule_case=RuleCase(
                description="broken", source="SELECT FROM WHERE (", expected_finding_count=0
            ),
            expected_error="could not parse the case SQL",
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_invalid_rule_case_when_evaluating_then_reports_the_reason(
    tmp_path: Path, test_case: HarnessFailureTestCase
) -> None:
    rule: RuleCheck = sample_rule(tmp_path, test_case.rule_name)

    with pytest.raises(SqlCleanError) as raised:
        evaluate_rule(rule=rule, test_case=test_case.rule_case)

    assert test_case.expected_error in str(raised.value)
