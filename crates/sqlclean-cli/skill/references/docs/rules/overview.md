<!-- generated-by: sqlclean skills -->

# Overview

> Select Rules, record reviewed exceptions, and apply safe and unsafe fixes.

Online: https://sqlclean.io/docs/rules/overview/

## Contents

- Selecting Rules
- Default Rules
- Thresholds
- Exceptions
- Path ignores
- Parse errors (SQC999)
- Safe and unsafe fixes
- Caching
- Continue

A Rule is one check over SQL. It reports a finding at an exact position, with a message, a
remediation, and sometimes a fix. SQLClean has 45 built-in Rules with `SQC###` codes, and runs
[custom Rules](../custom-rules/overview.md) written in Python with `XSQC` codes.

Consider a query that returns an arbitrary hundred rows:

```sql
SELECT order_id
FROM orders
LIMIT 100
```

```bash
sqlclean check
```

```text
error[SQC004]: Row selection is nondeterministic
  --> models/recent_orders.sql:3:1
   |
 3 | LIMIT 100
   | ^^^^^
  = help: Add ORDER BY with a deterministic tie-breaker before LIMIT or OFFSET.

Found 1 problem.
```

Add an ordering and check again:

```sql
SELECT order_id
FROM orders
ORDER BY order_id
LIMIT 100
```

Run `sqlclean rules` for every Rule, or see the [Rule reference](reference.md).

## Selecting Rules

Select Rules in `sqlclean.toml` by exact code or by prefix:

```toml
[rules]
select = ["SQC"]              # every built-in Rule
ignore = ["SQC027", "SQC03"]  # except SQC027 and SQC030 to SQC039
```

- An exact code such as `SQC004` selects one Rule.
- A prefix selects every Rule whose code starts with it: `SQC` selects all built-in Rules,
  `SQC00` selects `SQC001` to `SQC009`, and `XSQCNAME` selects every custom Rule in the `NAME`
  family.
- `ignore` removes Rules from the selection, by code or prefix.
- A selector that matches no Rule is an error: `Rule selector 'SQC9' matches no Rule`.

On the command line, `--select` replaces `[rules] select` for one run, and `--ignore` adds to
`[rules] ignore`:

```bash
sqlclean check --select SQC001,SQC004
sqlclean check --ignore SQC039
```

`select = []` runs no Rules. Files are still parsed, so [parse errors](#parse-errors-sqc999) are
still reported.

## Default Rules

Without `[rules] select`, SQLClean runs the 13 Rules that are enabled by default. They cover
mistakes that run without an error and are easy to miss in review: NULL handling, joins, row order,
duplicate names and unused CTEs.

| Code | Finding |
|------|---------|
| `SQC001` | Comparison with NULL is never true |
| `SQC002` | Comma-separated sources create an implicit cartesian join |
| `SQC003` | Non-cross join has no meaningful condition |
| `SQC004` | Row selection is nondeterministic |
| `SQC005` | CTE is unreachable from the final query |
| `SQC006` | DISTINCT is redundant with the grouped output |
| `SQC007` | Positional set operation is vulnerable to column-order drift |
| `SQC009` | Table alias is duplicated in the same query scope |
| `SQC010` | Output alias is duplicated in the same select list |
| `SQC018` | ROW_NUMBER has no deterministic ordering |
| `SQC019` | NOT IN list contains NULL and can reject every row |
| `SQC020` | Set-operation branches project different column counts |
| `SQC039` | Inline query relation obscures data flow |

The other 32 Rules run only when `select` names them, by code or prefix. A prefix selects every
matching Rule, whether it is enabled by default or not. A custom Rule is enabled by default when its
`@rule` sets `enabled_by_default=True`.

## Thresholds

Two built-in Rules compare against a limit, set under `[rules.thresholds]`:

```toml
[rules.thresholds]
max_literal_length = 100      # SQC044: the longest string literal allowed
max_ranking_order_by = 5      # SQC043: the most ORDER BY expressions in a ranking window
```

Both default to the values shown and must be positive. Built-in Rules take no other options.

## Exceptions

An exception allows one reviewed finding: one Rule code in one file. It lives in `sqlclean.toml`,
so a reviewer sees it in the diff, and it needs a reason:

```toml
[[rules.rule_exceptions]]
rule = "SQC004"
path = "models/order_preview.sql"
reason = "A preview of any hundred orders is enough."
```

`rule` is an exact code, not a prefix. `path` is the file's path relative to the configuration
root, as findings print it. The exception covers every finding of that Rule in that file.

SQLClean has no inline suppression comments. A comment such as `-- noqa: SQC001` changes nothing;
record the exception in `sqlclean.toml` instead.

### Cross join exceptions

A file-wide exception for a deliberate `CROSS JOIN` would also hide any accidental cross join added
to the file later. `SQC038` findings name the joined relation and, inside a CTE, that CTE, so an
exception can cover exactly one join:

```text
error[SQC038]: CROSS JOIN with `regions` in CTE `pairs` creates an explicit cartesian product
  --> models/calendar.sql:11:3
```

```toml
[[rules.rule_exceptions]]
rule = "SQC038"
path = "models/calendar.sql"
relation = "regions"
cte = "pairs"                 # optional: only the join inside this CTE
reason = "Every order date is reported for every region."
```

`relation` and `cte` compare without case. Without `cte`, the exception covers joins with that
relation anywhere in the file. `cte` needs `relation`, and `relation` matches only `SQC038`
findings.

`SQC038` does not report a cross join when one side is provably at most one row, such as a CTE
that aggregates without `GROUP BY`, `LIMIT 1` or a one-row `VALUES`, so many intentional cross
joins need no exception:

```sql
WITH latest AS (
  SELECT MAX(order_date) AS latest_date
  FROM orders
)

SELECT
  o.order_id,
  l.latest_date
FROM orders AS o
CROSS JOIN latest AS l
```

### Stale exceptions

An exception that matches no finding is stale. `check` and `fix` report it and exit with `1`:

```text
error: stale rule exception: SQC004 in models/orders.sql matches no finding; remove it from sqlclean.toml
```

Delete the entry. Exceptions are only judged stale when their file was checked and their Rule was
selected, so `sqlclean check models/customers.sql` or `--select SQC001` never reports an exception
for another file or Rule.

## Path ignores

A path ignore turns Rules off for files that follow a different convention. It takes codes or
prefixes, glob patterns, and a reason:

```toml
[[rules.rule_ignores]]
rules = ["SQC034", "SQC035"]
paths = ["scratch/**"]
reason = "Exploratory queries."
```

Patterns are relative to the configuration root, as in [`include`](../configuration.md#choosing-files).
To turn a Rule off everywhere, add it to `[rules] ignore` instead.

## Parse errors (SQC999)

A file that does not parse under its [dialect](../configuration.md#dialects) gets one `SQC999`
finding and no other:

```text
error[SQC999]: SQL could not be parsed as duckdb: Parse error at line 1, column 18: Expected table name or subquery, got Where
  --> scratch/broken.sql:1:1
   |
 1 | SELECT FROM WHERE orders
   | ^
  = help: Fix the syntax error, or set this file's dialect in sqlclean.toml.
```

The finding points at the start of the file; the message gives the line and column of the error.
`SQC999` is not a selectable Rule: selection, `ignore`, path ignores and exceptions never hide it.
Fix the SQL, set the right dialect for the file, or [`exclude`](../configuration.md#choosing-files)
the file.

## Safe and unsafe fixes

Some Rules carry a fix. [`sqlclean fix`](../cli/fix.md) applies it.

- A **safe** fix keeps the query's results. `SQC005` removes an unused CTE; `SQC002` turns
  `FROM orders, customers` into `FROM orders CROSS JOIN customers`.
- An **unsafe** fix can change results. `SQC001` turns `cancelled_at = NULL`, which matches no row,
  into `cancelled_at IS NULL`, which matches rows. That is almost always the intent, but it is a
  different query. Unsafe fixes apply only with `--unsafe-fixes`.

`sqlclean check` says which command applies each fix:

```text
  = fix: available with `sqlclean fix --unsafe-fixes`
```

The [Rule reference](reference.md) lists which Rules can fix and how safely.

## Caching

`check` and `fix` reuse the results of earlier runs for inputs that have not changed, so a second
run over an unchanged project costs little more than reading the files. Results are stored in
`.sqlclean/` under the configuration root, which has its own `.gitignore`.

A file's results are evaluated again when any of these change:

- the file's text, path or dialect;
- the selected Rules or `[rules.thresholds]`;
- a custom Rule's options, or any Python file under `[rules] custom`;
- for a custom Rule over the whole repository, any checked file;
- the SQLClean version.

Exceptions and ignores apply after the cache, so editing them takes effect at once.

Pass `--no-cache` to evaluate every file without reading or writing the cache. Use it when a custom
Rule imports Python from outside its `[rules] custom` paths, since changes there are not tracked.
`sqlclean format` does not use the cache.

## Continue

    Every built-in Rule, its default and its fix.
    Define your own Rules in Python.
    Every `sqlclean.toml` key.
    Flags, output formats and exit codes.
