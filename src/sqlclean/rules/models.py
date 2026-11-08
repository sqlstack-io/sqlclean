"""Structured values Rules receive and return."""

from dataclasses import dataclass, field

from sqlclean.rules.classes.rule_option import RuleOption
from sqlclean.rules.types import FixSafety, RuleCheck, RuleSubjectKind


@dataclass(frozen=True, slots=True)
class Edit:
    """Replace the characters start..end of a file with text."""

    start: int
    end: int
    text: str


@dataclass(frozen=True, slots=True)
class Finding:
    """One Rule violation located by character offsets in its file."""

    code: str
    path: str
    start: int
    end: int
    message: str
    remediation: str
    edits: tuple[Edit, ...] = ()


@dataclass(frozen=True, slots=True)
class Cte:
    """A common table expression, located at its name."""

    name: str
    text: str
    start: int
    end: int
    line: int
    column: int


@dataclass(frozen=True, slots=True)
class Table:
    """A relation read in FROM or JOIN, located at its name."""

    name: str
    qualified_name: str
    alias: str | None
    text: str
    start: int
    end: int
    line: int
    column: int


@dataclass(frozen=True, slots=True)
class Join:
    """One join, located at the relation it joins."""

    kind: str
    relation: str | None
    has_condition: bool
    text: str
    start: int
    end: int
    line: int
    column: int


@dataclass(frozen=True, slots=True)
class Star:
    """A * or qualifier.* projection."""

    qualifier: str | None
    text: str
    start: int
    end: int
    line: int
    column: int


@dataclass(frozen=True, slots=True)
class FunctionCall:
    """A call of a named function, located at its name."""

    name: str
    text: str
    start: int
    end: int
    line: int
    column: int


@dataclass(frozen=True, slots=True)
class ColumnReference:
    """A column reference with its qualifier when written."""

    name: str
    qualifier: str | None
    text: str
    start: int
    end: int
    line: int
    column: int


@dataclass(frozen=True, slots=True)
class Literal:
    """A string or numeric literal as written."""

    kind: str
    text: str
    start: int
    end: int
    line: int
    column: int


@dataclass(frozen=True)
class Statement:
    """One statement of a SQL file, located by character offsets in that file."""

    path: str
    index: int
    kind: str
    text: str
    start: int
    end: int
    line: int
    column: int
    dialect: str
    ctes: tuple[Cte, ...]
    tables: tuple[Table, ...]
    joins: tuple[Join, ...]
    stars: tuple[Star, ...]
    functions: tuple[FunctionCall, ...]
    columns: tuple[ColumnReference, ...]
    literals: tuple[Literal, ...]


@dataclass(frozen=True)
class SqlFile:
    """One SQL file and its statements."""

    path: str
    source: str
    dialect: str
    statements: tuple[Statement, ...]


@dataclass(frozen=True)
class Repository:
    """Every checked SQL file, for Rules about conventions across files."""

    files: tuple[SqlFile, ...]


@dataclass(frozen=True, slots=True)
class RuleDefinition:
    """The metadata @rule attaches to a Rule function."""

    code: str
    slug: str
    message: str
    remediation: str
    check: RuleCheck
    subject: RuleSubjectKind
    subject_parameter: str
    context_parameter: str
    options: tuple[RuleOption[object], ...] = ()
    enabled_by_default: bool = False
    fixable: FixSafety | None = None
    module: str = field(default="")
