"""Run one Rule over its subjects and check what it returns."""

import traceback
from collections.abc import Iterator, Mapping
from typing import cast

from sqlclean.hosting.exceptions import RuleEvaluationError
from sqlclean.rules.classes.rule_context import RuleContext
from sqlclean.rules.classes.rule_option import RuleOption
from sqlclean.rules.exceptions import RuleOptionError, SqlCleanError
from sqlclean.rules.models import Finding, Repository, RuleDefinition, SqlFile
from sqlclean.rules.types import RuleOptionValue, RuleSubjectKind


def evaluate(
    *,
    definition: RuleDefinition,
    files: tuple[SqlFile, ...],
    options: Mapping[str, object],
) -> tuple[Finding, ...]:
    """Return the Rule's findings over each of its subjects in files."""

    values: dict[str, RuleOptionValue] = parse_options(definition=definition, configured=options)
    sources: dict[str, str] = {file.path: file.source for file in files}
    findings: list[Finding] = []
    for subject, file in _subjects(definition=definition, files=files):
        context: RuleContext = RuleContext(definition=definition, options=values, file=file)
        returned: object = _call(definition=definition, subject=subject, context=context)
        findings.extend(_checked(definition=definition, returned=returned, sources=sources))
    return tuple(findings)


def parse_options(
    *, definition: RuleDefinition, configured: Mapping[str, object]
) -> dict[str, RuleOptionValue]:
    """Return the configured option values, validated against the Rule's declarations."""

    declared: dict[str, RuleOption[object]] = {option.name: option for option in definition.options}
    unknown: list[str] = sorted(set(configured) - set(declared))
    if unknown:
        raise RuleOptionError(f"Rule {definition.code} has no option {', '.join(unknown)}")
    values: dict[str, RuleOptionValue] = {}
    for option in definition.options:
        if option.name in configured:
            try:
                values[option.name] = cast(RuleOptionValue, option.parse(configured[option.name]))
            except RuleOptionError as error:
                raise RuleOptionError(f"[rules.rule_options.{definition.code}] {error}") from error
    return values


def _subjects(
    *, definition: RuleDefinition, files: tuple[SqlFile, ...]
) -> Iterator[tuple[object, SqlFile | None]]:
    if definition.subject is RuleSubjectKind.REPOSITORY:
        yield Repository(files=files), None
        return
    for file in files:
        if definition.subject is RuleSubjectKind.FILE:
            yield file, file
        else:
            for statement in file.statements:
                yield statement, file


def _call(*, definition: RuleDefinition, subject: object, context: RuleContext) -> object:
    arguments: dict[str, object] = {
        definition.subject_parameter: subject,
        definition.context_parameter: context,
    }
    try:
        return definition.check(**arguments)
    except SqlCleanError:
        raise
    except Exception as error:
        location: str = getattr(subject, "path", "the repository")
        raise RuleEvaluationError(
            f"custom Rule {definition.code} failed on {location}: "
            f"{type(error).__name__}: {error}\n{traceback.format_exc()}"
        ) from error


def _checked(
    *, definition: RuleDefinition, returned: object, sources: Mapping[str, str]
) -> list[Finding]:
    if not isinstance(returned, list | tuple) or not all(
        isinstance(item, Finding) for item in returned
    ):
        raise RuleEvaluationError(
            f"custom Rule {definition.code} must return a list of findings from ctx.finding(...)"
        )
    for finding in returned:
        _check_finding(definition=definition, finding=finding, sources=sources)
    return list(returned)


def _check_finding(
    *, definition: RuleDefinition, finding: Finding, sources: Mapping[str, str]
) -> None:
    source: str | None = sources.get(finding.path)
    if finding.code != definition.code or source is None:
        raise RuleEvaluationError(
            f"custom Rule {definition.code} returned a finding for another Rule or file"
        )
    for edit in finding.edits:
        if not 0 <= edit.start <= edit.end <= len(source):
            raise RuleEvaluationError(
                f"custom Rule {definition.code} returned an edit outside {finding.path}"
            )
