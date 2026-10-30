"""One host request: list the custom Rules, or evaluate them over files."""

from collections.abc import Mapping, Sequence
from pathlib import Path

from sqlclean.hosting._helpers.evaluation import evaluate
from sqlclean.hosting._helpers.loading import load_definitions
from sqlclean.hosting._helpers.sql_files import sql_file_from_facts
from sqlclean.hosting.constants import CATALOGUE_MODE
from sqlclean.hosting.exceptions import RuleEvaluationError
from sqlclean.rules.exceptions import SqlCleanError
from sqlclean.rules.models import Edit, Finding, RuleDefinition, SqlFile

type _Json = dict[str, object]


def handle_request(request: Mapping[str, object]) -> _Json:
    """Return the response to one request, or an error the command reports."""

    try:
        custom: list[str] = [str(path) for path in _list(container=request, key="custom")]
        definitions: dict[str, RuleDefinition] = load_definitions(
            root=Path(str(request["root"])), custom=custom
        )
        if request["mode"] == CATALOGUE_MODE:
            return {"rules": [_rule_info(definition) for definition in definitions.values()]}
        return {"results": _evaluate(request=request, definitions=definitions)}
    except SqlCleanError as error:
        return {"error": str(error)}


def _evaluate(
    *, request: Mapping[str, object], definitions: Mapping[str, RuleDefinition]
) -> list[_Json]:
    files: list[tuple[SqlFile, list[str]]] = [
        _requested_file(item) for item in _list(container=request, key="files")
    ]
    options: Mapping[str, object] = _mapping(container=request, key="options")
    results: list[_Json] = []
    for file, codes in files:
        for code in codes:
            findings: tuple[Finding, ...] = evaluate(
                definition=_definition(definitions=definitions, code=code),
                files=(file,),
                options=_mapping(container=options, key=code),
            )
            results.append(_result(code=code, path=file.path, findings=findings))
    every_file: tuple[SqlFile, ...] = tuple(file for file, _ in files)
    for code in _list(container=request, key="repository_rules"):
        findings = evaluate(
            definition=_definition(definitions=definitions, code=str(code)),
            files=every_file,
            options=_mapping(container=options, key=str(code)),
        )
        results.append(_result(code=str(code), path=None, findings=findings))
    return results


def _requested_file(item: object) -> tuple[SqlFile, list[str]]:
    fields: Mapping[str, object] = item if isinstance(item, Mapping) else {}
    file: SqlFile = sql_file_from_facts(
        path=str(fields["path"]),
        source=str(fields["source"]),
        dialect=str(fields["dialect"]),
        facts=_mapping(container=fields, key="facts"),
    )
    return file, [str(code) for code in _list(container=fields, key="rules")]


def _definition(*, definitions: Mapping[str, RuleDefinition], code: str) -> RuleDefinition:
    definition: RuleDefinition | None = definitions.get(code)
    if definition is None:
        raise RuleEvaluationError(f"custom Rule {code} is no longer defined")
    return definition


def _rule_info(definition: RuleDefinition) -> _Json:
    return {
        "code": definition.code,
        "message": definition.message,
        "remediation": definition.remediation,
        "enabled_by_default": definition.enabled_by_default,
        "custom": True,
        "subject": definition.subject.value,
        "fixable": None if definition.fixable is None else definition.fixable.value,
        "module": definition.module,
    }


def _result(*, code: str, path: str | None, findings: tuple[Finding, ...]) -> _Json:
    return {"code": code, "path": path, "findings": [_finding(finding) for finding in findings]}


def _finding(finding: Finding) -> _Json:
    return {
        "path": finding.path,
        "start": finding.start,
        "end": finding.end,
        "message": finding.message,
        "remediation": finding.remediation,
        "edits": [_edit(edit) for edit in finding.edits],
    }


def _edit(edit: Edit) -> _Json:
    return {"start": edit.start, "end": edit.end, "text": edit.text}


def _list(*, container: Mapping[str, object], key: str) -> Sequence[object]:
    value: object = container.get(key, ())
    return value if isinstance(value, list) else ()


def _mapping(*, container: Mapping[str, object], key: str) -> Mapping[str, object]:
    value: object = container.get(key, {})
    return value if isinstance(value, Mapping) else {}
