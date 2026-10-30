"""Stable constants for SQLClean docs reference generation."""

from pathlib import Path

REPOSITORY_ROOT: Path = Path(__file__).resolve().parents[2]
DEFAULT_DOCS_ROOT: Path = REPOSITORY_ROOT / "website/src/content/docs/docs"
DEFAULT_OUTPUT_DIR: Path = REPOSITORY_ROOT / "crates/sqlclean-cli/skill/references/docs"
SITE_BASE_URL: str = "https://sqlclean.io"
DOCS_BASE_URL: str = f"{SITE_BASE_URL}/docs"
DOCS_SLUG: str = "docs"
DOCS_URL_PATH: str = f"/{DOCS_SLUG}"
INDEX_PAGE: str = "index"
COMPONENT_TAG_END: str = ">"
GENERATED_MARKER: str = "<!-- generated-by: sqlclean skills -->"
INDEX_FILENAME: str = "CONTENTS.md"
FRONTMATTER_DELIMITER: str = "---"
TABLE_OF_CONTENTS_MIN_LINES: int = 100
UNGROUPED_LABEL: str = "Other pages"
