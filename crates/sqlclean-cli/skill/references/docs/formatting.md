<!-- generated-by: sqlclean skills -->

# Formatting

> One canonical layout for every SQL file, the line width and indent, and files that can't be formatted.

Online: https://sqlclean.io/docs/formatting/

## Contents

- Before and after
- The layout
- Statements
- Line width
- Indent
- Idempotency
- Formatting in CI
- When a file can't be formatted

`sqlclean format` prints every statement in one canonical layout. The only style options are the
line width and the indent, so every file in a project looks the same whoever wrote it.
[`sqlclean fix`](cli/fix.md) formats the files it reads too.

## Before and after

```sql
-- Revenue per customer
with paid as (select order_id, customer_id, amount from payments where status = 'paid' and amount > 0),
totals as (select customer_id, sum(amount) as revenue, count(*) as orders from paid group by customer_id)
select c.customer_id, c.customer_name, t.revenue, t.orders, case when t.revenue > 1000 then 'gold' when t.revenue > 100 then 'silver' else 'bronze' end as tier
from customers c left join totals t on c.customer_id = t.customer_id
order by t.revenue desc;
delete from orders where status='cancelled';select 1
```

```bash
sqlclean format
```

```sql
-- Revenue per customer
WITH paid AS (
    SELECT
        order_id,
        customer_id,
        amount
    FROM payments
    WHERE
        status = 'paid' AND amount > 0
),

totals AS (
    SELECT
        customer_id,
        SUM(amount) AS revenue,
        COUNT(*) AS orders
    FROM paid
    GROUP BY
        customer_id
)

SELECT
    c.customer_id,
    c.customer_name,
    t.revenue,
    t.orders,
    CASE
        WHEN t.revenue > 1000
        THEN 'gold'
        WHEN t.revenue > 100
        THEN 'silver'
        ELSE 'bronze'
    END AS tier
FROM customers c
LEFT JOIN totals t
    ON c.customer_id = t.customer_id
ORDER BY
    t.revenue DESC;

DELETE FROM orders WHERE status = 'cancelled';

SELECT 1
```

## The layout

- Clause keywords and operators such as `SELECT`, `LEFT JOIN`, `AND` and `DESC`, and function
  names such as `SUM` and `COUNT`, are upper case. Identifiers, aliases and literals keep the case
  they were written in.
- Each clause starts a line. Each indent step is four spaces; see [Indent](#indent).
- A `SELECT` list with two or more items puts one item per line. A single item stays on the
  `SELECT` line: `SELECT order_id`, `SELECT *`.
- `GROUP BY` and `ORDER BY` items go on the lines below the clause.
- A `WHERE` with one condition stays on the clause line: `WHERE amount > 0`. Several conditions
  go on an indented line below it.
- A join's `ON` condition goes on the next line, indented.
- Each CTE closes with `),` and one blank line. One blank line separates the last CTE from the
  query that reads it.
- `CASE` puts each `WHEN`, `THEN` and `ELSE` on its own line.
- Comments are kept.

## Statements

Each statement in a file is formatted on its own. Statements are separated by one blank line, and
every file ends with a newline. Semicolons are kept where they are written; the formatter does not
add one after the last statement.

## Line width

`line_width` sets the width the formatter fills before it breaks a line. The default is `100`:

```toml
[format]
line_width = 100
```

A line that does not fit is broken at its outermost structure. Inner parentheses that fit on
their line stay on one line. At the default width this `IN` list fits; at `line_width = 60` it is
broken:

```sql
SELECT order_id
FROM orders
WHERE status IN ('placed', 'paid', 'packed', 'shipped', 'delivered', 'returned')
```

```sql
SELECT order_id
FROM orders
WHERE
    status IN (
        'placed',
        'paid',
        'packed',
        'shipped',
        'delivered',
        'returned'
    )
```

`line_width` must be positive.

## Indent

`indent` sets the number of spaces per indent step. The default is `4`:

```toml
[format]
indent = 4
```

With `indent = 2`, `select order_id, amount from orders where amount > 0` formats as:

```sql
SELECT
  order_id,
  amount
FROM orders
WHERE amount > 0
```

`indent` must be between `1` and `8`.

## Idempotency

Formatting is idempotent: a formatted file formats to itself. Running `sqlclean format` twice
changes nothing the second time, so `sqlclean format --check` passes on any file
`sqlclean format` wrote:

```text
0 file(s) reformatted, 1 file(s) already formatted.
```

## Formatting in CI

`sqlclean format --check` exits with `1` and names every file that would change, without writing:

```text
Would reformat: models/open_orders.sql
1 file(s) would be reformatted, 0 file(s) already formatted.
```

`--diff` prints the changes instead. See [`sqlclean format`](cli/format.md) and
[Integrations](integrations.md).

## When a file can't be formatted

The formatter leaves a file unchanged when it cannot format it, usually because the file does not
parse under its [dialect](configuration.md#dialects). Template code such as `{{ col }}` does
not parse:

```text
j.sql: could not format: Parse error at line 1, column 16: Expected Colon, got RBrace ('}') near [{{ col }} from orders]
```

`sqlclean format` and `sqlclean fix` report the file on stderr and exit with `1`, and format every
other file as usual. Fix the SQL, set the file's dialect with
[`[[dialects]]`](configuration.md#dialects), or
[`exclude`](configuration.md#choosing-files) the file. `sqlclean check` reports the same file as
[`SQC999`](rules/overview.md#parse-errors-sqc999).
