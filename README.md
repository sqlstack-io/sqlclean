<p align="center">
  <img src="https://raw.githubusercontent.com/sqlstack-io/sqlclean/main/.github/sqlclean-logo.png" alt="SQLClean" width="100%">
</p>

<h2 align="center">The fastest SQL linter.</h2>

<p align="center">
  <strong>{{SPEED_LINE}}</strong> <a href="https://sqlclean.io/docs/benchmarks/">Benchmarks</a>
</p>

<p align="center">
  Catch SQL mistakes, apply safe fixes, and format consistently. Add your own rules in Python.
</p>

<p align="center">
  <a href="https://sqlclean.io">Website</a> ·
  <a href="https://sqlclean.io/docs/quickstart/">Get started</a> ·
  <a href="https://sqlclean.io/docs/">Docs</a> ·
  <a href="https://sqlclean.io/docs/benchmarks/">Benchmarks</a>
</p>

```bash
uv tool install sqlclean        # or: pip install sqlclean
```

<p align="center">
  <img src="https://raw.githubusercontent.com/sqlstack-io/sqlclean/main/.github/demos/check-fix.gif" alt="sqlclean check reports three findings, then sqlclean fix applies two fixes and formats the file" width="100%">
</p>

`sqlclean check` reports a bare `JOIN`, a `COUNT(1)` and a temporary table from a custom Rule.
`sqlclean fix` applies the two fixes and formats the file; the custom Rule's finding has no fix,
so it stays.

## Speed

{{SPEED_TABLE}}

The tools don't check the same things, so see the
[benchmarks](https://sqlclean.io/docs/benchmarks/) for coverage, the corpus, the method and how
to run it yourself.

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

## Quick start

```bash
sqlclean check     # report findings in every .sql file under the directory
sqlclean fix       # apply safe fixes, then format
sqlclean format    # rewrite files in the canonical layout
```

Add a `sqlclean.toml` to choose a dialect, files or Rules. See the
[quickstart](https://sqlclean.io/docs/quickstart/) and
[configuration](https://sqlclean.io/docs/configuration/).

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
