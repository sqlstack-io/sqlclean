"""Type declarations of the custom Rule authoring API."""

from collections.abc import Callable, Sequence
from enum import StrEnum
from typing import Protocol

type RuleOptionValue = bool | int | str | tuple[str, ...] | tuple[int, ...]
type RuleCheck = Callable[..., Sequence[object]]


class FixSafety(StrEnum):
    """Whether a Rule's fix keeps query results (safe) or may change them (unsafe)."""

    SAFE = "safe"
    UNSAFE = "unsafe"


class RuleSubjectKind(StrEnum):
    """The subject a Rule evaluates."""

    STATEMENT = "statement"
    FILE = "file"
    REPOSITORY = "repository"


class Located(Protocol):
    """Anything with a character range in its file, such as a statement or a SQL node."""

    @property
    def start(self) -> int: ...

    @property
    def end(self) -> int: ...
