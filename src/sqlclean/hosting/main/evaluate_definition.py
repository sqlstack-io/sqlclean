"""Evaluate one custom Rule over files or a repository."""

from collections.abc import Mapping

from sqlclean.hosting._helpers.evaluation import evaluate
from sqlclean.rules.models import Finding, RuleDefinition, SqlFile


def evaluate_definition(
    *,
    definition: RuleDefinition,
    files: tuple[SqlFile, ...],
    options: Mapping[str, object],
) -> tuple[Finding, ...]:
    """Return the Rule's findings: per statement or file, or once over every file."""

    return evaluate(definition=definition, files=files, options=options)
