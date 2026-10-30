<!-- generated-by: sqlclean skills -->

# Integrations

> Run SQLClean in CI with JSON or SARIF output, in pre-commit, and from editors.

Online: https://sqlclean.io/docs/integrations/

## Contents

- CI
- pre-commit
- Editors

`sqlclean check`, `fix` and `format` exit with `0` when there is nothing to report, `1` when there
are findings or files to change, and `2` when they cannot run. Any CI system, Git hook or editor
can gate on that.

## CI

Run `sqlclean check` and `sqlclean format --check` on every change:

```bash
sqlclean check
sqlclean format --check
```

Both read every matching file under the configuration root. `check` fails on findings and
[stale exceptions](rules/overview.md#stale-exceptions); `format --check` fails on files that
need formatting or can't be formatted.

### Code scanning with SARIF

`--output-format sarif` writes a SARIF 2.1.0 log that code-scanning tools show as annotations on
the changed lines. On GitHub Actions:

```yaml
name: sqlclean

on: [push, pull_request]

jobs:
  sqlclean:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      - run: pip install sqlclean
      - run: sqlclean format --check
      - run: sqlclean check --output-format sarif > sqlclean.sarif
      - uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: sqlclean.sarif
```

`sqlclean check` exits with `1` when it reports findings, so the upload step runs with
`if: always()`. The SARIF log lists findings only; stale exceptions still fail the step. See
[SARIF output](cli/check.md#sarif-output).

### JSON for other tools

`--output-format json` prints one document with every finding, its position, message, remediation
and fix safety, the stale exceptions, and a summary. See
[JSON output](cli/check.md#json-output). `sqlclean rules --json` describes every Rule a finding
can come from.

### Caching in CI

`check` stores results in `.sqlclean/` under the configuration root. Keep that directory between
runs to skip files that have not changed, or pass `--no-cache` to evaluate everything. See
[Caching](rules/overview.md#caching).

## pre-commit

SQLClean ships three [pre-commit](https://pre-commit.com/) hooks:

| Hook | Runs | Changes files |
|------|------|---------------|
| `sqlclean-check` | `sqlclean check` | No |
| `sqlclean-fix` | `sqlclean fix` | Yes: safe fixes, then formatting |
| `sqlclean-format` | `sqlclean format` | Yes: formatting |

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/sqlstack-io/sqlclean
    rev: v0.0.1
    hooks:
      - id: sqlclean-fix
      - id: sqlclean-format
```

Use `sqlclean-check` instead of `sqlclean-fix` where a hook must not change files.

The hooks run on staged `.sql` files and pass them to the command by name, so `exclude` in
`sqlclean.toml` still applies but `include` does not. They use the nearest `sqlclean.toml` at
or above the repository root. A hook that changes a file, or leaves a finding, fails the commit;
stage the changes and commit again.

## Editors

Editors can run `sqlclean format` on save, passing the saved file:

```bash
sqlclean format path/to/file.sql
```

The file is formatted with the dialect, `line_width` and `indent` of the nearest `sqlclean.toml` at or above
the editor's working directory. Use `sqlclean check --output-format json path/to/file.sql` to show
findings with their lines and columns. Lines and columns start at 1 and count characters.
