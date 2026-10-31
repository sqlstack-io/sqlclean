---
name: sqlclean
description: ALWAYS load this skill when linting, fixing or formatting SQL with SQLClean, editing sqlclean.toml, writing or testing custom Rules (Python @rule functions with XSQLCLEAN codes), recording Rule exceptions, or reading SQLClean findings (SQLCLEAN codes, SQLCLEAN999 parse errors) in CI, JSON or SARIF output.
---
<!-- generated-by: sqlclean skills -->

# SQLClean

SQLClean is a fast SQL linter and formatter. `sqlclean check` reports Rule findings,
`sqlclean fix` applies their fixes and formats, and `sqlclean format` rewrites files in one
canonical layout. Custom Rules are plain Python functions over structural facts that the Rust
engine extracts from each statement.

## The working loop

1. **Orient.** Read the nearest `sqlclean.toml` (dialect, include and exclude, `[format]`,
   `[rules]`). Paths in it are relative to its directory.
2. **Check.** `sqlclean check [PATHS]` exits 0 when clean, 1 with findings, 2 on errors. Use
   `--output-format json` when you need to read findings programmatically.
3. **Explain.** `sqlclean rules SQLCLEAN001` shows a Rule's message, fix safety and remediation.
4. **Fix.** `sqlclean fix --diff` previews safe fixes plus formatting; `sqlclean fix` writes them.
   Fixes marked unsafe can change query results, such as `= NULL` becoming `IS NULL`. Never pass
   `--unsafe-fixes` without the user's say-so.
5. **Format.** `sqlclean format --check` fails when a file needs formatting. Formatting is
   idempotent, so rerunning it changes nothing.
6. **Recheck** until `sqlclean check` exits 0.

## Rules of thumb

- Fix the SQL rather than silencing a finding. When a finding is genuinely intended, record a
  `[[rules.rule_exceptions]]` entry with the Rule, path and a `reason`. Use
  `[[rules.rule_ignores]]` only for whole paths a Rule does not apply to.
- An exception that no longer matches a finding is reported as stale; remove it.
- `SQLCLEAN999` is a parse error. It cannot be ignored: fix the SQL or set the right `dialect`,
  globally or per path under `[[dialects]]`.
- `--select` and `--ignore` take codes or prefixes such as `SQLCLEAN001` or `SQLCLEAN`.
- Results are cached in `.sqlclean/cache/`. Use `--no-cache` only when debugging a custom Rule
  that imports helpers from outside its `[rules] custom` paths.

## Custom Rules

- A custom Rule is a function decorated with `@rule` from `sqlclean`, in a file under a path
  listed in `[rules] custom`. Its code is `XSQLCLEAN`, optional capital letters, then three digits.
- The first keyword parameter names the subject: `statement: Statement`, `file: SqlFile` or
  `repository: Repository`. The second is `ctx: RuleContext`.
- Return `ctx.finding(...)` values. Fixes are lists of `ctx.replace`, `ctx.insert_before`,
  `ctx.insert_after` and `ctx.delete` edits, and the Rule declares `fixable`.
- Never print to standard output inside a Rule: the host speaks JSON over it.
- Test Rules with `sqlclean.testing.evaluate_rule` and `RuleCase`.
- `sqlclean facts --dialect duckdb < query.sql` prints the facts a Rule sees for a statement.

## Exact syntax

The bundled documentation in `references/docs/` matches the installed SQLClean version. Start
from `references/docs/CONTENTS.md` and open only the page you need:

- Configuration keys: `references/docs/configuration.md`
- Every built-in Rule: `references/docs/rules/reference.md`
- Writing custom Rules: `references/docs/custom-rules/overview.md` and
  `references/docs/custom-rules/statement-facts.md`
- Command flags: `references/docs/cli/`
