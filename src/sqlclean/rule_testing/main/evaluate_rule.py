"""Run one custom Rule over a test case."""

from sqlclean.rule_testing._helpers.harness import run_case
from sqlclean.rule_testing.models import RuleCase, RuleResult
from sqlclean.rules.types import RuleCheck


def evaluate_rule(*, rule: RuleCheck, test_case: RuleCase) -> RuleResult:
    """Return the Rule's findings on the case, failing when the count or determinism differs."""

    return run_case(rule=rule, test_case=test_case)
