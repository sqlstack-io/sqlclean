"""Turn SQLBuild's compiled DuckShop 10K models into plain Snowflake SQL files.

Usage: python convert_duckshop.py COMPILED_MODELS OUTDIR

COMPILED_MODELS is `target/compiled/models` after `sqb compile` in a project written by SQLBuild's
`benchmarks/duckshop-10k/generate.py`. Compiling has already removed the MODEL headers and expanded
the macros; what is left of SQLBuild is its reference functions, which become ordinary names:

| Compiled SQL | Plain SQL |
|---|---|
| `__ref("orders")` | `orders` |
| `__source("ledger__journals")` | `raw.ledger__journals` |
| `__seed("source_instances")` | `seeds.source_instances` |
| `__udf("luhn_valid")` | `luhn_valid` |
| `__table_fn("customer_orders")(...)` | `TABLE(customer_orders(...))` |
| `__cursor_start()`, `__cursor_end()` | fixed timestamps |

OUTDIR/sql/ receives one file per model in the same folders, plus a configuration for each linter
that selects the Snowflake dialect and nothing else, so each tool runs its default Rules.
"""

import argparse
import re
from pathlib import Path

REPLACEMENTS: tuple[tuple[re.Pattern[str], str], ...] = (
    (re.compile(r'__table_fn\("([^"]+)"\)\(([^()]*)\)'), r"TABLE(\1(\2))"),
    (re.compile(r'__ref\("([^"]+)"\)'), r"\1"),
    (re.compile(r'__source\("([^"]+)"\)'), r"raw.\1"),
    (re.compile(r'__seed\("([^"]+)"\)'), r"seeds.\1"),
    (re.compile(r'__udf\("([^"]+)"\)'), r"\1"),
    (re.compile(r"__cursor_start\(\)"), "'2026-01-01 00:00:00'::TIMESTAMP_NTZ"),
    (re.compile(r"__cursor_end\(\)"), "'2026-01-02 00:00:00'::TIMESTAMP_NTZ"),
)
LEFTOVER: re.Pattern[str] = re.compile(r"\b__[a-z_]+\(")
CONFIGS: dict[str, str] = {
    "sqlclean.toml": 'dialect = "snowflake"\n',
    ".sqruff": "[sqruff]\ndialect = snowflake\n",
    ".sqlfluff": "[sqlfluff]\ndialect = snowflake\n",
}


def plain_sql(text: str) -> str:
    """Return text with every SQLBuild reference function replaced by plain SQL."""

    for pattern, replacement in REPLACEMENTS:
        text = pattern.sub(replacement, text)
    leftover: re.Match[str] | None = LEFTOVER.search(text)
    if leftover is not None:
        raise SystemExit(f"unconverted SQLBuild function: {leftover.group(0)}")
    return text


def main() -> None:
    """Convert every compiled model and write the linter configurations."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("compiled", type=Path)
    parser.add_argument("out", type=Path)
    arguments = parser.parse_args()
    sql: Path = arguments.out / "sql"
    files: list[Path] = sorted(arguments.compiled.rglob("*.sql"))
    for source in files:
        target: Path = sql / source.relative_to(arguments.compiled)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(plain_sql(source.read_text(encoding="utf-8")), encoding="utf-8")
    for name, text in CONFIGS.items():
        (sql / name).write_text(text, encoding="utf-8")
    print(f"wrote {len(files)} SQL files to {sql}")


if __name__ == "__main__":
    main()
