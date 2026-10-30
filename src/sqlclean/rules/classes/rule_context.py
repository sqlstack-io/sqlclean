"""What a Rule receives besides its subject: options, findings and fix edits."""

from collections.abc import Mapping, Sequence
from typing import cast

from sqlclean.rules.classes.rule_option import RuleOption
from sqlclean.rules.exceptions import RuleUsageError
from sqlclean.rules.models import Edit, Finding, RuleDefinition, SqlFile, Statement
from sqlclean.rules.types import Located, RuleOptionValue


class RuleContext:
    """The per-evaluation API of one Rule: typed options, findings and edits."""

    def __init__(
        self,
        *,
        definition: RuleDefinition,
        options: Mapping[str, RuleOptionValue],
        file: SqlFile | None,
    ) -> None:
        self._definition: RuleDefinition = definition
        self._options: Mapping[str, RuleOptionValue] = options
        self._file: SqlFile | None = file

    @property
    def file(self) -> SqlFile | None:
        """Return the file being checked, or None for a Repository subject."""

        return self._file

    @property
    def dialect(self) -> str | None:
        """Return the dialect of the file being checked, or None for a Repository subject."""

        return None if self._file is None else self._file.dialect

    def option[T](self, option: RuleOption[T]) -> T:
        """Return the configured value of one declared option, or its default."""

        if all(declared is not option for declared in self._definition.options):
            raise RuleUsageError(
                f"Rule {self._definition.code} reads undeclared option {option.name}"
            )
        if option.name not in self._options:
            return option.default
        return cast(T, self._options[option.name])

    def finding(
        self,
        *,
        subject: Statement | SqlFile,
        node: Located | None = None,
        message: str | None = None,
        remediation: str | None = None,
        fix: Sequence[Edit] = (),
    ) -> Finding:
        """Return a finding at node, or at the subject, with an optional fix."""

        if fix and self._definition.fixable is None:
            raise RuleUsageError(
                f"Rule {self._definition.code} returned a fix but does not declare fixable"
            )
        located: Located = node if node is not None else _subject_location(subject)
        return Finding(
            code=self._definition.code,
            path=subject.path,
            start=located.start,
            end=located.end,
            message=message or self._definition.message,
            remediation=remediation or self._definition.remediation,
            edits=tuple(fix),
        )

    def replace(self, *, node: Located, text: str) -> Edit:
        """Return an edit that replaces node's text."""

        return Edit(start=node.start, end=node.end, text=text)

    def insert_before(self, *, node: Located, text: str) -> Edit:
        """Return an edit that inserts text immediately before node."""

        return Edit(start=node.start, end=node.start, text=text)

    def insert_after(self, *, node: Located, text: str) -> Edit:
        """Return an edit that inserts text immediately after node."""

        return Edit(start=node.end, end=node.end, text=text)

    def delete(self, *, node: Located) -> Edit:
        """Return an edit that removes node's text."""

        return Edit(start=node.start, end=node.end, text="")


def _subject_location(subject: Statement | SqlFile) -> Located:
    if isinstance(subject, SqlFile):
        return Edit(start=0, end=0, text="")
    return subject
