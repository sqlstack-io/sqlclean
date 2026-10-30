<!-- generated-by: sqlclean skills -->

# Introduction

> Clean SQL, every time. Fast linting, safe fixes and one canonical format.

Online: https://sqlclean.io/docs/

SQLClean is a free, open-source SQL linter and formatter. It checks `.sql` files against built-in
Rules, applies their fixes, and formats every file in one canonical layout. The engine and the
`sqlclean` command are written in Rust. Custom Rules are plain Python functions that read
structural facts about each statement.

```bash
sqlclean check
```

```text
error[SQC001]: Comparison with NULL is never true
  --> models/open_orders.sql:1:77
   |
 1 | with open_orders as (select order_id, amount from orders where cancelled_at = null),
   |                                                                             ^
  = help: Use IS NULL or IS NOT NULL when testing for NULL.
  = fix: available with `sqlclean fix --unsafe-fixes`

error[SQC005]: CTE is unreachable from the final query
  --> models/open_orders.sql:2:1
   |
 2 | refunds as (select order_id from payments where amount < 0)
   | ^^^^^^^
  = help: Reference the CTE from the final query or another reachable CTE, or remove it.
  = fix: available with `sqlclean fix`

Found 2 problems (1 fixable with `sqlclean fix`, 1 more with `--unsafe-fixes`).
```

`sqlclean fix` removes the unused CTE, then formats the file. The `= NULL` comparison has a fix
too, but it changes which rows match, so it waits for `--unsafe-fixes`.

## Check

[`sqlclean check`](cli/check.md) reports findings with the source line, the problem and what to
do about it. Output is text for people, or [JSON and SARIF](integrations.md) for CI and code
scanning. The [built-in Rules](rules/reference.md) cover SQL that runs but returns the wrong
rows: `NULL` comparisons, implicit cartesian joins, `LIMIT` without `ORDER BY`, unused CTEs and
mismatched set operations. Opt-in Rules enforce a consistent query structure.

## Fix

[`sqlclean fix`](cli/fix.md) applies every safe fix until the files stop changing, then formats
them. A fix that can change query results is marked unsafe and runs only with `--unsafe-fixes`.
See [Safe and unsafe fixes](rules/overview.md#safe-and-unsafe-fixes).

## Format

[`sqlclean format`](cli/format.md) rewrites every file in one canonical layout. Keywords are
upper case, each clause starts a line, and statements are separated by one blank line. Formatting
is idempotent: a formatted file formats to itself. See [Formatting](formatting.md).

## Extend

[Custom Rules](custom-rules/overview.md) are Python functions decorated with `@rule`. They read
the tables, joins, CTEs, columns and literals of each statement, can carry fixes and typed options,
and run with the same selection, exceptions and cache as the built-in Rules. Tools that compile or
template SQL can [embed the engine](embedding.md) from Rust.

## Commands

| Command | What it does |
|---------|--------------|
| [`sqlclean check`](cli/check.md) | Reports Rule findings without changing any file |
| [`sqlclean fix`](cli/fix.md) | Applies Rule fixes until stable, then formats |
| [`sqlclean format`](cli/format.md) | Rewrites SQL files in the canonical layout |
| [`sqlclean rules`](cli/rules.md) | Lists the available Rules, or shows one |
| [`sqlclean facts`](cli/facts.md) | Prints the statement facts custom Rules read, as JSON |

## Where to start

    Install SQLClean, then check, fix and format your first file.
    Selection, defaults, exceptions and fixes.
    Encode your team's SQL conventions in Python.
    Every command and flag.
