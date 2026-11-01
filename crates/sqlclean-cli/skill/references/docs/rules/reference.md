<!-- generated-by: sqlclean skills -->

# Rule reference

> Every built-in Rule, whether it runs by default, and whether it has a fix.

Online: https://sqlclean.io/docs/rules/reference/

## Contents

- SQLCLEAN001: NULL comparisons
- SQLCLEAN002 and SQLCLEAN003: joins without keys
- SQLCLEAN005: unused CTEs
- SQLCLEAN018: unstable row numbers
- SQLCLEAN019: NULL in NOT IN
- SQLCLEAN046 to SQLCLEAN053: queries that cannot do what they say
- SQLCLEAN039: inline subqueries
- SQLCLEAN034, SQLCLEAN035 and SQLCLEAN041: query structure

SQLClean has 53 built-in Rules. `Default` Rules run when `[rules] select` is not set; the others
run when `select` names them by code or prefix. A Rule with a `Safe` fix is fixed by
`sqlclean fix`; an `Unsafe` fix also needs `--unsafe-fixes`. See
[Selecting Rules](overview.md#selecting-rules) and
[Safe and unsafe fixes](overview.md#safe-and-unsafe-fixes).

Run `sqlclean rules <CODE>` for a Rule's remediation.

| Code | Finding | Default | Fix |
|------|---------|---------|-----|
| `SQLCLEAN001` | Comparison with NULL is never true | Yes | Unsafe |
| `SQLCLEAN002` | Comma-separated sources create an implicit cartesian join | Yes | Safe |
| `SQLCLEAN003` | Non-cross join has no meaningful condition | Yes | Safe |
| `SQLCLEAN004` | Row selection is nondeterministic | Yes | — |
| `SQLCLEAN005` | CTE is unreachable from the final query | Yes | Safe |
| `SQLCLEAN006` | DISTINCT is redundant with the grouped output | Yes | Safe |
| `SQLCLEAN007` | Positional set operation is vulnerable to column-order drift | Yes | — |
| `SQLCLEAN008` | Bare UNION leaves duplicate handling implicit | No | Safe |
| `SQLCLEAN009` | Table alias is duplicated in the same query scope | Yes | — |
| `SQLCLEAN010` | Output alias is duplicated in the same select list | Yes | — |
| `SQLCLEAN011` | Clause mixes positional and named references | No | — |
| `SQLCLEAN012` | ELSE NULL is redundant | No | Safe |
| `SQLCLEAN013` | DISTINCT is written as though it were a function | No | Safe |
| `SQLCLEAN014` | Predicate compares identical constant values | No | — |
| `SQLCLEAN015` | Empty statement follows another terminator | No | Safe |
| `SQLCLEAN016` | Expression is aliased to its existing name | No | — |
| `SQLCLEAN017` | Row count uses a constant expression | No | Safe |
| `SQLCLEAN018` | ROW_NUMBER has no deterministic ordering | Yes | — |
| `SQLCLEAN019` | NOT IN list contains NULL and can reject every row | Yes | — |
| `SQLCLEAN020` | Set-operation branches project different column counts | Yes | — |
| `SQLCLEAN021` | Query output has an uncontrolled column shape | No | — |
| `SQLCLEAN022` | Calculated projection has no stable output name | No | — |
| `SQLCLEAN023` | Table alias is never referenced | No | Safe |
| `SQLCLEAN024` | WHERE predicate rejects NULL rows from a LEFT JOIN | No | — |
| `SQLCLEAN025` | JOIN type is implicit | No | Safe |
| `SQLCLEAN026` | ORDER BY mixes explicit and implicit directions | No | — |
| `SQLCLEAN027` | Column reference is unqualified in a multi-relation query | No | — |
| `SQLCLEAN028` | Single-relation query mixes qualified and unqualified columns | No | — |
| `SQLCLEAN029` | Qualified reference uses a relation absent from this query scope | No | — |
| `SQLCLEAN030` | CASE expression only converts a condition to TRUE or FALSE | No | Safe |
| `SQLCLEAN031` | CASE expression is nested directly inside ELSE | No | — |
| `SQLCLEAN032` | Joined relation contributes no referenced values | No | — |
| `SQLCLEAN033` | Comment is not attached to the syntax node it explains | No | — |
| `SQLCLEAN034` | Query keeps transformation logic outside top-level CTEs | No | — |
| `SQLCLEAN035` | Terminal SELECT does not read plainly from the final top-level CTE | No | — |
| `SQLCLEAN036` | CTE is nested inside another query | No | — |
| `SQLCLEAN037` | Recursive CTE is not permitted | No | — |
| `SQLCLEAN038` | CROSS JOIN creates an explicit cartesian product | No | — |
| `SQLCLEAN039` | Inline query relation obscures data flow | Yes | — |
| `SQLCLEAN040` | JOIN ON contains a computed expression | No | — |
| `SQLCLEAN041` | Only the last top-level CTE may be named final, and it must use that name | No | — |
| `SQLCLEAN042` | CTE output column is never read later in the query | No | Safe |
| `SQLCLEAN043` | Ranking window sorts by more expressions than max_ranking_order_by | No | — |
| `SQLCLEAN044` | String literal is longer than max_literal_length | No | — |
| `SQLCLEAN045` | JSON payload is parsed more than once in one SELECT | No | — |
| `SQLCLEAN046` | WITH defines the same CTE name more than once | Yes | — |
| `SQLCLEAN047` | CASE WHEN NULL or IN (NULL) never matches | Yes | — |
| `SQLCLEAN048` | Expression divides by a literal zero | Yes | — |
| `SQLCLEAN049` | UPDATE or DELETE has no limiting predicate and changes every row | No | — |
| `SQLCLEAN050` | INSERT does not list its target columns | No | — |
| `SQLCLEAN051` | NATURAL JOIN matches on every column the relations share | Yes | — |
| `SQLCLEAN052` | BETWEEN lower bound is greater than its upper bound | Yes | — |
| `SQLCLEAN053` | Aggregate or window function is used in WHERE, ON or GROUP BY | Yes | — |

`SQLCLEAN999` is the [parse error](overview.md#parse-errors-sqlclean999) finding. It is not a
selectable Rule.

`SQLCLEAN043` and `SQLCLEAN044` read their limits from
[`[rules.thresholds]`](overview.md#thresholds).

## SQLCLEAN001: NULL comparisons

`= NULL` and `<> NULL` are never true, so the filter matches no row:

```sql
SELECT order_id
FROM orders
WHERE cancelled_at = NULL
```

```text
error[SQLCLEAN001]: Comparison with NULL is never true
  --> models/open_orders.sql:3:20
   |
 3 | WHERE cancelled_at = NULL
   |                    ^
  = help: Use IS NULL or IS NOT NULL when testing for NULL.
  = fix: available with `sqlclean fix --unsafe-fixes`
```

The fix writes `WHERE cancelled_at IS NULL`. It is unsafe because the query then returns rows it
did not return before.

## SQLCLEAN002 and SQLCLEAN003: joins without keys

A comma between sources joins every row with every row unless a `WHERE` predicate filters them:

```sql
SELECT
  o.order_id,
  c.customer_name
FROM orders AS o, customers AS c
WHERE o.customer_id = c.customer_id
```

The `SQLCLEAN002` fix keeps the meaning and makes the cartesian product visible:

```diff
-FROM orders AS o, customers AS c
+FROM orders AS o
+CROSS JOIN customers AS c
 WHERE o.customer_id = c.customer_id
```

Move the predicate into `JOIN customers AS c ON o.customer_id = c.customer_id` by hand to make it a
keyed join. `SQLCLEAN003` reports a `JOIN` whose condition is missing or constant, such as
`ON 1 = 1`; its fix turns a `JOIN` without a condition into `CROSS JOIN`.

## SQLCLEAN005: unused CTEs

A CTE that no reachable query reads is dead code:

```sql
WITH paid AS (
  SELECT order_id FROM payments
),

refunded AS (
  SELECT order_id FROM refunds
)

SELECT order_id FROM paid
```

```text
error[SQLCLEAN005]: CTE is unreachable from the final query
  --> models/paid_orders.sql:5:1
   |
 5 | refunded AS (
   | ^^^^^^^^
  = help: Reference the CTE from the final query or another reachable CTE, or remove it.
  = fix: available with `sqlclean fix`
```

The fix removes `refunded`.

## SQLCLEAN018: unstable row numbers

Without `ORDER BY` in the window, the row numbered `1` can change between runs:

```sql
SELECT
  order_id,
  ROW_NUMBER() OVER (PARTITION BY customer_id) AS order_rank
FROM orders
```

```text
error[SQLCLEAN018]: ROW_NUMBER has no deterministic ordering
```

An `ORDER BY` is not always enough. Ties still rank in any order unless `PARTITION BY` and
`ORDER BY` together cover a unique key of the ranked rows. SQLClean only knows a key it can prove
from the query, such as the `GROUP BY` columns of a CTE. Ranking straight over a table is reported
with a second message:

```text
error[SQLCLEAN018]: Ranking window order is not proven unique within its partition
  --> models/ranked.sql:3:3
   |
 3 |   ROW_NUMBER() OVER (PARTITION BY customer_id ORDER BY ordered_at, order_id) AS order_rank
   |   ^^^^^^^^^^
  = help: PARTITION BY and ORDER BY must together cover a unique key of the ranked rows. End the ORDER BY with a unique, non-null tie-breaker column. If the rows have no unique column, build one upstream from the columns that identify a row (for example `HASH(source_partition, source_offset, line_index) AS row_key`) and order by it last.
```

Here the CTE's `GROUP BY` makes `customer_id, order_id` a key, so the window passes:

```sql
WITH customer_orders AS (
  SELECT
    customer_id,
    order_id,
    MAX(ordered_at) AS ordered_at
  FROM orders
  GROUP BY
    customer_id,
    order_id
)

SELECT
  order_id,
  ROW_NUMBER() OVER (PARTITION BY customer_id ORDER BY ordered_at, order_id) AS order_rank
FROM customer_orders
```

`SQLCLEAN004` reports the same problem for `LIMIT` and `OFFSET`; see the example in
[Rules](overview.md). It skips a query that selects only constants, such as
`SELECT 1 AS present FROM orders LIMIT 1`, because every row it could return is identical.

## SQLCLEAN019: NULL in NOT IN

`x NOT IN (..., NULL)` is never true, so the filter rejects every row:

```sql
SELECT order_id
FROM orders
WHERE status NOT IN ('cancelled', NULL)
```

```text
error[SQLCLEAN019]: NOT IN list contains NULL and can reject every row
  --> models/active_orders.sql:3:14
   |
 3 | WHERE status NOT IN ('cancelled', NULL)
   |              ^^^
  = help: Remove NULL from the list or express the anti-join with a NULL-safe NOT EXISTS.
```

## SQLCLEAN046 to SQLCLEAN053: queries that cannot do what they say

These Rules report a query that runs differently from how it reads. None of them has a fix,
because only the author knows the intended query.

```sql
WITH paid AS (
  SELECT order_id, amount FROM orders WHERE amount BETWEEN 100 AND 10  -- SQLCLEAN052
),

paid AS (                                                              -- SQLCLEAN046
  SELECT order_id, amount / 0 AS ratio FROM orders                     -- SQLCLEAN048
)

SELECT
  customer_id,
  CASE region WHEN NULL THEN 'unknown' END AS region_label             -- SQLCLEAN047
FROM paid
NATURAL JOIN customers                                                 -- SQLCLEAN051
WHERE SUM(amount) > 100                                                -- SQLCLEAN053
GROUP BY customer_id
```

- `SQLCLEAN046` compares CTE names within one `WITH` the way the dialect resolves them, so quoted
  `"paid"` and `"PAID"` are different names in PostgreSQL but the same name in DuckDB. A nested
  `WITH` may reuse an outer name.
- `SQLCLEAN047` reports `CASE x WHEN NULL` and `x IN (NULL)`, which compare with `=` and so never
  match. `NOT IN` lists with NULL are `SQLCLEAN019`.
- `SQLCLEAN048` reports `/`, `%`, `//` and `MOD` with a literal zero divisor. Depending on the dialect
  the query fails or returns NULL.
- `SQLCLEAN052` compares number and `DATE 'YYYY-MM-DD'` bounds exactly and skips `BETWEEN SYMMETRIC`.
  A reversed `NOT BETWEEN` is reported too, because it matches every non-NULL value.
- `SQLCLEAN053` reports aggregates in `WHERE`, `ON` and `GROUP BY`, and window functions anywhere in
  those clauses. Aggregates in `HAVING`, windows in `QUALIFY`, `FILTER (WHERE ...)` and
  subqueries pass.

Two write-safety Rules are opt-in, because changing every row is sometimes intended:

- `SQLCLEAN049` reports an `UPDATE` or `DELETE` whose rows nothing limits. A `WHERE` without a column
  or subquery, such as `WHERE TRUE`, does not count; a join condition or a `LIMIT` does.
- `SQLCLEAN050` reports `INSERT` without a column list. `DEFAULT VALUES` and DuckDB's `BY NAME` pass.

## SQLCLEAN039: inline subqueries

A subquery in `FROM` hides a step of the query. Name it as a CTE instead:

```sql
-- Reported
SELECT order_id
FROM (SELECT order_id FROM orders WHERE amount > 0) AS paid

-- Passes
WITH paid AS (
  SELECT order_id
  FROM orders
  WHERE amount > 0
)

SELECT order_id
FROM paid
```

## SQLCLEAN034, SQLCLEAN035 and SQLCLEAN041: query structure

These opt-in Rules ask every query to do its work in top-level CTEs, name the last one `final`,
and end with a plain `SELECT` of named columns from it. This query passes every built-in Rule:

```sql
WITH open_orders AS (
  SELECT
    order_id,
    amount
  FROM orders
  WHERE cancelled_at IS NULL
),

final AS (
  SELECT
    order_id,
    amount
  FROM open_orders
)

SELECT
  order_id,
  amount
FROM final
```

Ending with `SELECT * FROM final` is reported by `SQLCLEAN021`, which asks for named output columns.
