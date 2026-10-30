"""Import the configured custom Rule modules and collect their @rule functions."""

import sys
from importlib import import_module
from pathlib import Path
from types import ModuleType

from sqlclean.hosting.constants import (
    PYTHON_SUFFIX,
    SKIPPED_DIRECTORIES,
    SKIPPED_MODULE_NAMES,
    SKIPPED_MODULE_PREFIX,
)
from sqlclean.hosting.exceptions import RuleLoadingError
from sqlclean.rules.constants import RULE_ATTRIBUTE
from sqlclean.rules.models import RuleDefinition


def load_definitions(*, root: Path, custom: list[str]) -> dict[str, RuleDefinition]:
    """Return every custom Rule defined in the modules under the configured paths, by code."""

    root_text: str = str(root)
    if root_text not in sys.path:
        sys.path.insert(0, root_text)
    definitions: dict[str, RuleDefinition] = {}
    for module_name in _module_names(root=root, custom=custom):
        module: ModuleType = _import(module_name)
        for value in vars(module).values():
            definition: object = getattr(value, RULE_ATTRIBUTE, None)
            if not isinstance(definition, RuleDefinition) or definition.module != module_name:
                continue
            if definition.code in definitions:
                raise RuleLoadingError(f"custom Rule code {definition.code} is defined twice")
            definitions[definition.code] = definition
    return definitions


def _module_names(*, root: Path, custom: list[str]) -> list[str]:
    files: set[Path] = set()
    for entry in custom:
        path: Path = root / entry
        candidates: list[Path] = (
            [path] if path.is_file() else sorted(path.rglob(f"*{PYTHON_SUFFIX}"))
        )
        files.update(candidate for candidate in candidates if _is_rule_module(candidate))
    return sorted(".".join(file.relative_to(root).with_suffix("").parts) for file in files)


def _is_rule_module(path: Path) -> bool:
    return (
        path.suffix == PYTHON_SUFFIX
        and path.name not in SKIPPED_MODULE_NAMES
        and not path.name.startswith(SKIPPED_MODULE_PREFIX)
        and not SKIPPED_DIRECTORIES.intersection(path.parts)
    )


def _import(module_name: str) -> ModuleType:
    try:
        return import_module(module_name)
    except Exception as error:
        raise RuleLoadingError(
            f"could not import custom Rule module {module_name}: {type(error).__name__}: {error}"
        ) from error
