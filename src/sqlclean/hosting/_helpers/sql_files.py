"""Statement facts, with character offsets, turned into located Rule subjects."""

from bisect import bisect_right
from collections.abc import Mapping, Sequence
from typing import TypedDict

from sqlclean.hosting.constants import NEWLINE
from sqlclean.rules.models import (
    ColumnReference,
    Cte,
    FunctionCall,
    Join,
    Literal,
    SqlFile,
    Star,
    Statement,
    Table,
)

type _Facts = Mapping[str, object]


class _Location(TypedDict):
    text: str
    start: int
    end: int
    line: int
    column: int


class _Locator:
    def __init__(self, source: str) -> None:
        self.source: str = source
        self._starts: list[int] = [0] + [
            index + 1 for index, character in enumerate(source) if character == NEWLINE
        ]

    def located(self, facts: _Facts) -> _Location:
        start: int = _integer(facts=facts, key="start")
        end: int = _integer(facts=facts, key="end")
        line: int = bisect_right(self._starts, start)
        return _Location(
            text=self.source[start:end],
            start=start,
            end=end,
            line=line,
            column=start - self._starts[line - 1] + 1,
        )

    def cte(self, item: _Facts) -> Cte:
        return Cte(name=str(item["name"]), **self.located(item))

    def table(self, item: _Facts) -> Table:
        return Table(
            name=str(item["name"]),
            qualified_name=str(item["qualified_name"]),
            alias=_optional(facts=item, key="alias"),
            **self.located(item),
        )

    def join(self, item: _Facts) -> Join:
        return Join(
            kind=str(item["kind"]),
            relation=_optional(facts=item, key="relation"),
            has_condition=bool(item["has_condition"]),
            **self.located(item),
        )

    def star(self, item: _Facts) -> Star:
        return Star(qualifier=_optional(facts=item, key="qualifier"), **self.located(item))

    def function(self, item: _Facts) -> FunctionCall:
        return FunctionCall(name=str(item["name"]), **self.located(item))

    def column(self, item: _Facts) -> ColumnReference:
        return ColumnReference(
            name=str(item["name"]),
            qualifier=_optional(facts=item, key="qualifier"),
            **self.located(item),
        )

    def literal(self, item: _Facts) -> Literal:
        return Literal(kind=str(item["kind"]), **self.located(item))


def sql_file_from_facts(*, path: str, source: str, dialect: str, facts: _Facts) -> SqlFile:
    """Return the SqlFile for one file's native facts."""

    locator: _Locator = _Locator(source)
    statements: tuple[Statement, ...] = tuple(
        _statement(path=path, dialect=dialect, facts=statement, locator=locator)
        for statement in _items(facts=facts, key="statements")
    )
    return SqlFile(path=path, source=source, dialect=dialect, statements=statements)


def _statement(*, path: str, dialect: str, facts: _Facts, locator: _Locator) -> Statement:
    return Statement(
        path=path,
        index=_integer(facts=facts, key="index"),
        kind=str(facts["kind"]),
        dialect=dialect,
        ctes=tuple(locator.cte(item) for item in _items(facts=facts, key="ctes")),
        tables=tuple(locator.table(item) for item in _items(facts=facts, key="tables")),
        joins=tuple(locator.join(item) for item in _items(facts=facts, key="joins")),
        stars=tuple(locator.star(item) for item in _items(facts=facts, key="stars")),
        functions=tuple(locator.function(item) for item in _items(facts=facts, key="functions")),
        columns=tuple(locator.column(item) for item in _items(facts=facts, key="columns")),
        literals=tuple(locator.literal(item) for item in _items(facts=facts, key="literals")),
        **locator.located(facts),
    )


def _items(*, facts: _Facts, key: str) -> Sequence[_Facts]:
    value: object = facts.get(key, ())
    return value if isinstance(value, list) else ()


def _integer(*, facts: _Facts, key: str) -> int:
    value: object = facts[key]
    return value if isinstance(value, int) else 0


def _optional(*, facts: _Facts, key: str) -> str | None:
    value: object = facts.get(key)
    return None if value is None else str(value)
