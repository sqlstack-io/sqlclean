<!-- generated-by: sqlclean skills -->

# Quickstart

> Install SQLClean, then check, fix and format your first SQL file.

Online: https://sqlclean.io/docs/quickstart/

## Contents

- Prerequisites
- 1. Check a file
- 2. Fix
- 3. Format
- 4. Add a sqlclean.toml
- Next steps

This guide installs SQLClean, checks one SQL file, fixes what can be fixed, and adds a
`sqlclean.toml` that selects more Rules.

## Prerequisites

- Python 3.12 or newer
- SQLClean installed from [PyPI](https://pypi.org/project/sqlclean/):

```bash
uv tool install sqlclean        # or: pip install sqlclean
sqlclean --version
```

The `sqlclean` package installs the native `sqlclean` executable. Custom Rules run in the same
Python environment as that executable.

## 1. Check a file

Save this query as `models/open_orders.sql`:

```sql
with open_orders as (select order_id, customer_id, amount from orders where cancelled_at = null),
refunds as (select order_id from payments where amount < 0)
select order_id, customer_id, amount from open_orders limit 100
```

Run `sqlclean check` in the project directory. With no paths, it reads every `.sql` file under the
directory:

```bash
sqlclean check
```

```text
error[SQC001]: Comparison with NULL is never true
  --> models/open_orders.sql:1:90
   |
 1 | with open_orders as (select order_id, customer_id, amount from orders where cancelled_at = null),
   |                                                                                          ^
  = help: Use IS NULL or IS NOT NULL when testing for NULL.
  = fix: available with `sqlclean fix --unsafe-fixes`

error[SQC005]: CTE is unreachable from the final query
  --> models/open_orders.sql:2:1
   |
 2 | refunds as (select order_id from payments where amount < 0)
   | ^^^^^^^
  = help: Reference the CTE from the final query or another reachable CTE, or remove it.
  = fix: available with `sqlclean fix`

error[SQC004]: Row selection is nondeterministic
  --> models/open_orders.sql:3:55
   |
 3 | select order_id, customer_id, amount from open_orders limit 100
   |                                                       ^^^^^
  = help: Add ORDER BY with a deterministic tie-breaker before LIMIT or OFFSET.

Found 3 problems (1 fixable with `sqlclean fix`, 1 more with `--unsafe-fixes`).
```

Each finding names its Rule code, the file, line and column, and what to do. The command exits
with `1` because there are findings.

## 2. Fix

Preview the safe fixes as a diff, then apply them:

```bash
sqlclean fix --diff
sqlclean fix
```

```text
Fixed 1 problem(s) and changed 1 file(s).
error[SQC001]: Comparison with NULL is never true
  --> models/open_orders.sql:7:24
   |
 7 |     WHERE cancelled_at = NULL
   |                        ^
  = help: Use IS NULL or IS NOT NULL when testing for NULL.
  = fix: available with `sqlclean fix --unsafe-fixes`

error[SQC004]: Row selection is nondeterministic
  --> models/open_orders.sql:15:1
    |
 15 | LIMIT 100
    | ^^^^^
  = help: Add ORDER BY with a deterministic tie-breaker before LIMIT or OFFSET.

Found 2 problems (1 fixable with `sqlclean fix --unsafe-fixes`).
```

`fix` removed the unused `refunds` CTE and formatted the file. The `= NULL` fix turns the
comparison into `IS NULL`, which changes which rows match, so it is unsafe. Review it, then apply
it:

```bash
sqlclean fix --unsafe-fixes
```

`SQC004` has no fix: only you know which column makes the order deterministic. Add
`ORDER BY order_id` before `LIMIT 100`, then run `sqlclean fix` once more to format the new
clause. The file is now:

```sql
WITH open_orders AS (
    SELECT
        order_id,
        customer_id,
        amount
    FROM orders
    WHERE cancelled_at IS NULL
)

SELECT
    order_id,
    customer_id,
    amount
FROM open_orders
ORDER BY
    order_id
LIMIT 100
```

```bash
sqlclean check
```

```text
All checks passed.
```

## 3. Format

`sqlclean fix` formats the files it reads. To format without applying fixes, or to check
formatting in CI, use `sqlclean format`:

```bash
sqlclean format            # rewrite files
sqlclean format --check    # exit 1 when a file needs formatting
```

See [Formatting](formatting.md) for the layout.

## 4. Add a sqlclean.toml

Without a configuration file, SQLClean parses files as `generic` SQL and runs the 13 Rules that
are enabled by default. Create `sqlclean.toml` in the project root to set the dialect and select
every built-in Rule:

```toml
dialect = "duckdb"

[rules]
select = ["SQC"]
```

`SQC` is a prefix, so it selects all 45 built-in Rules. Three of the opt-in structure Rules now
report the file:

```text
error[SQC041]: Only the last top-level CTE may be named final, and it must use that name
  --> models/open_orders.sql:1:6
   |
 1 | WITH open_orders AS (
   |      ^^^^^^^^^^^
  = help: Name the last top-level CTE final and rename any other final CTE; SQC035 checks the terminal SELECT.

error[SQC034]: Query keeps transformation logic outside top-level CTEs
  --> models/open_orders.sql:10:1
    |
 10 | SELECT
    | ^^^^^^
  = help: Move transformation logic into named top-level CTEs before the terminal SELECT.

error[SQC035]: Terminal SELECT does not read plainly from the final top-level CTE
  --> models/open_orders.sql:10:1
    |
 10 | SELECT
    | ^^^^^^
  = help: Name the final logical CTE and leave one terminal SELECT from that CTE.

Found 3 problems.
```

Follow them, or skip the ones your team does not want:

```toml
dialect = "duckdb"

[rules]
select = ["SQC"]
ignore = ["SQC034", "SQC035", "SQC041"]
```

```text
All checks passed.
```

## Next steps

- [Configuration](configuration.md) lists every key, including per-path dialects.
- [Rules](rules/overview.md) covers selection, exceptions and fixes, and the
  [Rule reference](rules/reference.md) lists every built-in Rule.
- [Integrations](integrations.md) runs SQLClean in CI and pre-commit.
- [Custom Rules](custom-rules/overview.md) checks your own conventions.
