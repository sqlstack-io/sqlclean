"""One typed configuration option a Rule declares."""

from typing import cast

from sqlclean.rules.exceptions import RuleDefinitionError, RuleOptionError


class RuleOption[T]:
    """A named, typed option configured under [rules.rule_options.<code>]."""

    def __init__(
        self,
        *,
        name: str,
        value_type: type,
        default: T,
        description: str,
        choices: tuple[T, ...] = (),
        minimum: int | None = None,
        maximum: int | None = None,
        item_type: type | None = None,
    ) -> None:
        if not name.isidentifier():
            raise RuleDefinitionError(f"Rule option name must be an identifier: {name!r}")
        self.name: str = name
        self.value_type: type = value_type
        self.default: T = default
        self.description: str = description
        self.choices: tuple[T, ...] = choices
        self.minimum: int | None = minimum
        self.maximum: int | None = maximum
        self.item_type: type | None = item_type
        _ = self.parse(default)

    @classmethod
    def boolean(cls, *, name: str, default: bool, description: str) -> "RuleOption[bool]":
        """Declare a true or false option."""

        return RuleOption[bool](
            name=name, value_type=bool, default=default, description=description
        )

    @classmethod
    def integer(
        cls,
        *,
        name: str,
        default: int,
        description: str,
        minimum: int | None = None,
        maximum: int | None = None,
    ) -> "RuleOption[int]":
        """Declare a whole-number option with optional bounds."""

        return RuleOption[int](
            name=name,
            value_type=int,
            default=default,
            description=description,
            minimum=minimum,
            maximum=maximum,
        )

    @classmethod
    def string(
        cls, *, name: str, default: str, description: str, choices: tuple[str, ...] = ()
    ) -> "RuleOption[str]":
        """Declare a text option, optionally limited to choices."""

        return RuleOption[str](
            name=name, value_type=str, default=default, description=description, choices=choices
        )

    @classmethod
    def string_list(
        cls, *, name: str, default: tuple[str, ...], description: str
    ) -> "RuleOption[tuple[str, ...]]":
        """Declare a list-of-text option."""

        return RuleOption[tuple[str, ...]](
            name=name, value_type=tuple, default=default, description=description, item_type=str
        )

    @classmethod
    def integer_list(
        cls, *, name: str, default: tuple[int, ...], description: str
    ) -> "RuleOption[tuple[int, ...]]":
        """Declare a list-of-whole-numbers option."""

        return RuleOption[tuple[int, ...]](
            name=name, value_type=tuple, default=default, description=description, item_type=int
        )

    def parse(self, value: object) -> T:
        """Return a configured value as this option's type, or raise RuleOptionError."""

        if self.item_type is not None:
            return self._parse_list(value)
        if not self._is_scalar(value=value, expected=self.value_type):
            raise RuleOptionError(
                f"option {self.name} expects {self.value_type.__name__}, got {value!r}"
            )
        if self.choices and value not in self.choices:
            raise RuleOptionError(f"option {self.name} must be one of {list(self.choices)!r}")
        if isinstance(value, int) and not isinstance(value, bool):
            self._check_bounds(value)
        return cast(T, value)

    def _parse_list(self, value: object) -> T:
        if not isinstance(value, list | tuple):
            raise RuleOptionError(f"option {self.name} expects a list, got {value!r}")
        items: tuple[object, ...] = tuple(value)
        item_type: type = self.item_type or str
        if not all(self._is_scalar(value=item, expected=item_type) for item in items):
            raise RuleOptionError(f"option {self.name} expects a list of {item_type.__name__}")
        return cast(T, items)

    def _check_bounds(self, value: int) -> None:
        if self.minimum is not None and value < self.minimum:
            raise RuleOptionError(f"option {self.name} must be at least {self.minimum}")
        if self.maximum is not None and value > self.maximum:
            raise RuleOptionError(f"option {self.name} must be at most {self.maximum}")

    @staticmethod
    def _is_scalar(*, value: object, expected: type) -> bool:
        if expected is int and isinstance(value, bool):
            return False
        return isinstance(value, expected)
