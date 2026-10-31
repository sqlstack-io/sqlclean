<p align="center">
  <img src="https://raw.githubusercontent.com/sqlstack-io/sqlclean/main/.github/sqlclean-logo.png" alt="SQLClean" width="100%">
</p>

<p align="center">
  <strong>Clean SQL, every time.</strong> Fast linting, safe fixes and one canonical format.
</p>

<p align="center">
  <a href="https://sqlclean.io">Website</a> ·
  <a href="https://sqlclean.io/docs/">Docs</a> ·
  <a href="https://sqlclean.io/docs/quickstart/">Quickstart</a> ·
  <a href="https://sqlclean.io/docs/benchmarks/">Benchmarks</a>
</p>

SQLClean is a SQL linter and formatter. It flags SQL that runs but rarely does what you meant,
applies the fixes that are safe, and formats every file one way. The engine and the `sqlclean`
command are written in Rust. Custom Rules are plain Python functions, backed by a Rust API that
parses the SQL for them. It is free and open source under Apache-2.0.

<p align="center">
  <img src="https://raw.githubusercontent.com/sqlstack-io/sqlclean/main/.github/demos/check-fix.gif" alt="sqlclean check reports three findings, then sqlclean fix applies two fixes and formats the file" width="100%">
</p>

`sqlclean check` reports a bare `JOIN`, a `COUNT(1)` and a temporary table from a custom Rule.
`sqlclean fix` applies the two fixes and formats the file; the custom Rule's finding has no fix,
so it stays.

## Quick start

```bash
uv tool install sqlclean        # or: pip install sqlclean
sqlclean check                  # report findings in every .sql file under the directory
sqlclean fix                    # apply safe fixes, then format
sqlclean format                 # rewrite files in the canonical layout
```

Add a `sqlclean.toml` when you want to choose a dialect, files or Rules. See the
[quickstart](https://sqlclean.io/docs/quickstart/) and
[configuration](https://sqlclean.io/docs/configuration/).

## What it does

### Catch SQL that is easy to get wrong

- **Built-in Rules.** Default Rules flag comparisons with `NULL`, `NOT IN` lists that contain
  `NULL`, `ROW_NUMBER` with no deterministic order and more. Each finding points at the exact
  characters and says what to write instead. See [Rules](https://sqlclean.io/docs/rules/overview/)
  and the [Rule reference](https://sqlclean.io/docs/rules/reference/).
- **Reviewed exceptions.** An exception allows one Rule in one file, with a reason. When the SQL
  changes and it no longer matches, `sqlclean check` fails until it is removed. See
  [exceptions](https://sqlclean.io/docs/rules/overview/#exceptions).

### Fix and format safely

- **Safe fixes by default.** `sqlclean fix` applies safe fixes until the file is stable, then
  formats it. A fix that can change which rows match, such as `= NULL` to `IS NULL`, waits for
  `--unsafe-fixes`, and a fix that would break parsing is thrown away. See
  [`sqlclean fix`](https://sqlclean.io/docs/cli/fix/).
- **One canonical format.** Every statement gets the same layout, and formatting a formatted file
  changes nothing. See [formatting](https://sqlclean.io/docs/formatting/).

### Make your conventions Rules

- **Custom Rules in plain Python.** A Rule is a function over the tables, joins, columns and
  literals of each statement, already parsed and located by the Rust engine. No regular
  expressions and no parser to learn. See [custom Rules](https://sqlclean.io/docs/custom-rules/overview/).
- **Tested like any other code.** `sqlclean.testing` runs a Rule with the same engine as
  `sqlclean check`. See [testing custom Rules](https://sqlclean.io/docs/custom-rules/testing/).

### Run it everywhere

- **CI and pre-commit.** `sqlclean check` exits non-zero on findings and writes SARIF for
  code-scanning annotations, or JSON for anything else. See
  [integrations](https://sqlclean.io/docs/integrations/).
- **Agent skills.** `sqlclean skills` installs a skill that teaches AI coding agents the commands,
  Rules and configuration of the installed version. See
  [`sqlclean skills`](https://sqlclean.io/docs/cli/skills/).
- **An embeddable engine.** Tools that template or compile SQL can call the Rust crate directly.
  See [embedding](https://sqlclean.io/docs/embedding/).

## Speed

On 10,282 Snowflake SQL files, on a Linux machine with 4 CPUs and no cache, `sqlclean check` with
its 13 default Rules took 0.81 s, against 41 s for `sqruff lint` and 919 s for
`sqlfluff lint --processes 4`. All 45 built-in Rules took 1.33 s, and the default Rules plus three
custom Python Rules took 2.27 s. A rerun with nothing changed reads the result cache and takes
0.23 s. These are SQLClean 0.0.1 measurements, each tool with its default rules; the tools don't
check the same things. See the [benchmarks](https://sqlclean.io/docs/benchmarks/) for the method,
the corpus and how to run it yourself.

## pre-commit

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/sqlstack-io/sqlclean
    rev: v0.1.0
    hooks:
      - id: sqlclean-fix
      - id: sqlclean-format
```

Use `sqlclean-check` instead of `sqlclean-fix` where a hook must not change files. See
[pre-commit](https://sqlclean.io/docs/integrations/#pre-commit).

## Development

```bash
uv sync --all-groups   # builds the native executable into .venv
make check             # format, lint, type-check, fensu and clippy
make test              # Rust and Python tests
```

## License

SQLClean is licensed under the
[Apache License 2.0](https://github.com/sqlstack-io/sqlclean/blob/main/LICENSE).
