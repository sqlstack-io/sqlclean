<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/sqlstack-io/sqlclean/main/.github/sqlclean-logo-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/sqlstack-io/sqlclean/main/.github/sqlclean-logo-light.png">
    <img src="https://raw.githubusercontent.com/sqlstack-io/sqlclean/main/.github/sqlclean-logo-light.png" alt="SQLClean" width="100%">
  </picture>
</p>

<h2 align="center">The fastest SQL linter.</h2>

<p align="center">
  <strong>50× faster than sqruff. Lint 10,000 SQL files in under a second.</strong> <a href="https://sqlclean.io/docs/benchmarks/">Benchmarks</a>
  <br>
  <a href="https://sqlclean.io">Website</a> ·
  <a href="https://sqlclean.io/docs/quickstart/">Get started</a> ·
  <a href="https://sqlclean.io/docs/">Docs</a> ·
  <a href="https://sqlclean.io/docs/benchmarks/">Benchmarks</a>
</p>

Catch SQL mistakes, apply safe fixes, and format consistently. Add your own rules in Python.
SQLClean parses and checks SQL in Rust, and runs your Python Rules on statements it has already
parsed.

```bash
uv tool install sqlclean        # or: pip install sqlclean
```

<p align="center">
  <img src="https://raw.githubusercontent.com/sqlstack-io/sqlclean/main/.github/demos/check-fix.gif" alt="sqlclean check reports an unused CTE, a temporary table from a custom Rule and a comparison with NULL; sqlclean fix removes the CTE and formats the file; sqlclean fix --unsafe-fixes rewrites = NULL to IS NULL" width="100%">
</p>

`sqlclean check` reports an unused CTE, a temporary table from a custom Rule and a `= NULL`
comparison that is never true. `sqlclean fix` applies the safe fix, removing the CTE, and formats
the file. It leaves `= NULL` alone because rewriting it to `IS NULL` changes which rows match;
`sqlclean fix --unsafe-fixes` applies that fix. The custom Rule's finding has no fix, so it stays.

## Speed

Linting the same 10,282 Snowflake SQL files on 4 CPUs, with no cache:

| Command | Wall time | Time relative to `sqlclean check` |
|---|---:|---:|
| `sqlclean check` (default Rules) | **0.81 s** | 1× |
| `sqlclean check --select SQC` (all Rules) | **1.33 s** | 1.6× |
| `sqlclean check` with three custom Python Rules | **2.27 s** | 2.8× |
| `sqruff lint` | 41.0 s | 51× |
| `sqlfluff lint --processes 4` | 919 s | 1,135× |

Run again with nothing changed, `sqlclean check` reads its result cache and finishes in 0.23 s.

The tools don't check the same things, so see the
[benchmarks](https://sqlclean.io/docs/benchmarks/) for coverage, the corpus, the method and how
to run it yourself.

## Quick start

```bash
sqlclean check     # report findings in every .sql file under the directory
sqlclean fix       # apply safe fixes, then format
sqlclean format    # rewrite files in the canonical layout
```

Add a `sqlclean.toml` to choose a dialect, files or Rules. See the
[quickstart](https://sqlclean.io/docs/quickstart/) and
[configuration](https://sqlclean.io/docs/configuration/).

## Correct and safe

- **Rules for SQL that runs but is wrong.** Built-in Rules flag comparisons with `NULL`, `NOT IN`
  lists that contain `NULL`, `ROW_NUMBER` with no deterministic order and more. See
  [Rules](https://sqlclean.io/docs/rules/overview/).
- **Safe fixes by default.** `sqlclean fix` applies safe fixes only. A fix that can change which
  rows match waits for `--unsafe-fixes`, and a fix that would break parsing is discarded. See
  [`sqlclean fix`](https://sqlclean.io/docs/cli/fix/).
- **An idempotent formatter.** Every statement gets one canonical layout, and formatting a
  formatted file changes nothing. See [formatting](https://sqlclean.io/docs/formatting/).

## Custom Rules in Python

A custom Rule is a Python function over the tables, joins, columns and literals of each
statement, already parsed and located by the Rust engine:

```python
# sql_rules/naming.py
from sqlclean import Finding, RuleContext, Statement, rule


@rule(
    code="XSQCNAME001",
    message="Query reads a temporary table",
    remediation="Read from the published table instead.",
    enabled_by_default=True,
)
def temporary_table(*, statement: Statement, ctx: RuleContext) -> list[Finding]:
    return [
        ctx.finding(subject=statement, node=table)
        for table in statement.tables
        if table.name.startswith("tmp_")
    ]
```

Point `[rules] custom = ["sql_rules"]` in `sqlclean.toml` at it. See
[custom Rules](https://sqlclean.io/docs/custom-rules/overview/) and
[testing custom Rules](https://sqlclean.io/docs/custom-rules/testing/).

## Integrations

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/sqlstack-io/sqlclean
    rev: v0.1.0
    hooks:
      - id: sqlclean-fix
      - id: sqlclean-format
```

Use `sqlclean-check` where a hook must not change files. In CI, `sqlclean check` exits non-zero
on findings and writes SARIF with `--output-format sarif` for code-scanning annotations. See
[integrations](https://sqlclean.io/docs/integrations/). `sqlclean skills` installs a skill for AI
coding agents; see [`sqlclean skills`](https://sqlclean.io/docs/cli/skills/).

## Docs

- [Docs home](https://sqlclean.io/docs/)
- [Rule reference](https://sqlclean.io/docs/rules/reference/)
- [Configuration](https://sqlclean.io/docs/configuration/)
- [Embedding the Rust engine](https://sqlclean.io/docs/embedding/)

## Development

```bash
uv sync --all-groups   # builds the native executable into .venv
make check             # format, lint, type-check, fensu and clippy
make test              # Rust and Python tests
```

## License

[Apache License 2.0](https://github.com/sqlstack-io/sqlclean/blob/main/LICENSE).
