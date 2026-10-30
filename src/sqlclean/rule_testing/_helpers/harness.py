"""Evaluate a Rule over a case twice, in opposite file orders, and compare."""

from sqlclean.hosting.main.build_sql_file import build_sql_file
from sqlclean.hosting.main.evaluate_definition import evaluate_definition
from sqlclean.rule_testing._helpers.native_facts import native_facts
from sqlclean.rule_testing.exceptions import RuleCaseError
from sqlclean.rule_testing.models import RuleCase, RuleFile, RuleResult
from sqlclean.rules.constants import RULE_ATTRIBUTE
from sqlclean.rules.models import Edit, Finding, RuleDefinition, SqlFile
from sqlclean.rules.types import RuleCheck


def run_case(*, rule: RuleCheck, test_case: RuleCase) -> RuleResult:
    """Return the Rule's result on the case after checking count and determinism."""

    definition: object = getattr(rule, RULE_ATTRIBUTE, None)
    if not isinstance(definition, RuleDefinition):
        raise RuleCaseError("evaluate_rule needs a function decorated with @rule")
    files: tuple[SqlFile, ...] = tuple(
        build_sql_file(
            path=file.path,
            source=file.source,
            dialect=test_case.dialect,
            facts=native_facts(source=file.source, dialect=test_case.dialect),
        )
        for file in (RuleFile(path=test_case.path, source=test_case.source), *test_case.files)
    )
    first: tuple[Finding, ...] = evaluate_definition(
        definition=definition, files=files, options=test_case.options
    )
    second: tuple[Finding, ...] = evaluate_definition(
        definition=definition, files=tuple(reversed(files)), options=test_case.options
    )
    if sorted(first, key=_order) != sorted(second, key=_order):
        raise RuleCaseError(f"custom Rule {definition.code} is not deterministic")
    findings: tuple[Finding, ...] = tuple(sorted(first, key=_order))
    if len(findings) != test_case.expected_finding_count:
        raise RuleCaseError(
            f"{test_case.description}: expected {test_case.expected_finding_count} findings, "
            f"got {len(findings)}"
        )
    return RuleResult(
        findings=findings, fixed_source=_fixed(test_case=test_case, findings=findings)
    )


def _order(finding: Finding) -> tuple[str, int, int, str]:
    return finding.path, finding.start, finding.end, finding.message


def _fixed(*, test_case: RuleCase, findings: tuple[Finding, ...]) -> str:
    edits: list[Edit] = []
    for finding in findings:
        if finding.path == test_case.path:
            edits.extend(finding.edits)
    edits.sort(key=lambda edit: (edit.start, edit.end))
    pieces: list[str] = []
    cursor: int = 0
    for edit in edits:
        if edit.start < cursor:
            continue
        pieces.append(test_case.source[cursor : edit.start])
        pieces.append(edit.text)
        cursor = edit.end
    pieces.append(test_case.source[cursor:])
    return "".join(pieces)
