"""The @rule decorator that declares a custom Rule."""

from collections.abc import Callable

from sqlclean.rules._helpers.definition import define_rule
from sqlclean.rules.classes.rule_option import RuleOption
from sqlclean.rules.types import FixSafety, RuleCheck


def rule(
    *,
    code: str,
    message: str,
    remediation: str,
    slug: str | None = None,
    options: tuple[RuleOption[object], ...] = (),
    enabled_by_default: bool = False,
    fixable: FixSafety | str | None = None,
) -> Callable[[RuleCheck], RuleCheck]:
    """Declare one custom Rule while returning the function unchanged."""

    return define_rule(
        code=code,
        slug=slug if slug is not None else code.lower(),
        message=message,
        remediation=remediation,
        options=options,
        enabled_by_default=enabled_by_default,
        fixable=None if fixable is None else FixSafety(fixable),
    )
