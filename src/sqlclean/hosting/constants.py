"""Names in the JSON the sqlclean command exchanges with this host."""

CATALOGUE_MODE: str = "catalogue"
EVALUATE_MODE: str = "evaluate"
NEWLINE: str = "\n"
PYTHON_SUFFIX: str = ".py"
SKIPPED_MODULE_PREFIX: str = "test_"
SKIPPED_MODULE_NAMES: frozenset[str] = frozenset({"conftest.py", "__init__.py"})
SKIPPED_DIRECTORIES: frozenset[str] = frozenset({"__pycache__"})
