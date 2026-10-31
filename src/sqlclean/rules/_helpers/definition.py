"""Validate a @rule declaration and attach its definition to the function."""

import inspect
from collections.abc import Callable
from typing import get_type_hints

from sqlclean.rules.classes.rule_context import RuleContext
from sqlclean.rules.classes.rule_option import RuleOption
from sqlclean.rules.constants import (
    CUSTOM_CODE_PREFIX,
    RULE_ATTRIBUTE,
    RULE_CODE_DIGIT_COUNT,
    RULE_PARAMETER_COUNT,
    SIGNATURE_REQUIREMENT,
)
from sqlclean.rules.exceptions import RuleDefinitionError
from sqlclean.rules.models import Repository, RuleDefinition, SqlFile, Statement
from sqlclean.rules.types import FixSafety, RuleCheck, RuleSubjectKind

_SUBJECTS: dict[type, RuleSubjectKind] = {
    Statement: RuleSubjectKind.STATEMENT,
    SqlFile: RuleSubjectKind.FILE,
    Repository: RuleSubjectKind.REPOSITORY,
}


def define_rule(
    *,
    code: str,
    slug: str,
    message: str,
    remediation: str,
    options: tuple[RuleOption[object], ...],
    enabled_by_default: bool,
    fixable: FixSafety | None,
) -> Callable[[RuleCheck], RuleCheck]:
    """Return the decorator that validates and attaches one Rule definition."""

    if not all(value.strip() for value in (code, slug, message, remediation)):
        raise RuleDefinitionError(f"custom Rule {code} needs a code, slug, message and remediation")
    if not is_custom_code(code):
        raise RuleDefinitionError(
            f"custom Rule code must be {CUSTOM_CODE_PREFIX}, optional capital letters and "
            f"{RULE_CODE_DIGIT_COUNT} digits, such as XSQLCLEANNAME001: {code}"
        )
    names: list[str] = [option.name for option in options]
    if len(names) != len(set(names)):
        raise RuleDefinitionError(f"custom Rule {code} declares an option name twice")

    def decorate(check: RuleCheck) -> RuleCheck:
        subject, subject_parameter, context_parameter = _signature(code=code, check=check)
        definition: RuleDefinition = RuleDefinition(
            code=code,
            slug=slug,
            message=message,
            remediation=remediation,
            check=check,
            subject=subject,
            subject_parameter=subject_parameter,
            context_parameter=context_parameter,
            options=options,
            enabled_by_default=enabled_by_default,
            fixable=fixable,
            module=check.__module__,
        )
        setattr(check, RULE_ATTRIBUTE, definition)
        return check

    return decorate


def is_custom_code(code: str) -> bool:
    """Return whether code is XSQLCLEAN, optional capital letters and three digits."""

    family: str = code[:-RULE_CODE_DIGIT_COUNT]
    number: str = code[-RULE_CODE_DIGIT_COUNT:]
    letters: str = family.removeprefix(CUSTOM_CODE_PREFIX)
    return (
        family.startswith(CUSTOM_CODE_PREFIX)
        and (not letters or (letters.isascii() and letters.isalpha() and letters.isupper()))
        and number.isascii()
        and number.isdecimal()
        and len(number) == RULE_CODE_DIGIT_COUNT
    )


def _signature(*, code: str, check: RuleCheck) -> tuple[RuleSubjectKind, str, str]:
    parameters: tuple[inspect.Parameter, ...] = tuple(inspect.signature(check).parameters.values())
    if len(parameters) != RULE_PARAMETER_COUNT or any(
        parameter.kind is not inspect.Parameter.KEYWORD_ONLY for parameter in parameters
    ):
        raise RuleDefinitionError(f"custom Rule {code} {SIGNATURE_REQUIREMENT}")
    try:
        hints: dict[str, object] = get_type_hints(check)
    except (NameError, TypeError) as error:
        raise RuleDefinitionError(
            f"could not resolve custom Rule {code} annotations: {error}"
        ) from error
    subjects: list[tuple[RuleSubjectKind, str]] = [
        (_SUBJECTS[hint], name)
        for name, hint in hints.items()
        if isinstance(hint, type) and hint in _SUBJECTS
    ]
    contexts: list[str] = [name for name, hint in hints.items() if hint is RuleContext]
    if len(subjects) != 1 or len(contexts) != 1:
        raise RuleDefinitionError(f"custom Rule {code} {SIGNATURE_REQUIREMENT}")
    return subjects[0][0], subjects[0][1], contexts[0]
