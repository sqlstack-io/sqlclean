<!-- generated-by: sqlclean skills -->

# Configuration

> Every sqlclean.toml key, its default, per-path dialects and how the file is found.

Online: https://sqlclean.io/docs/configuration/

## Contents

- Finding the file
- Keys
- Choosing files
- Dialects
- Validation

SQLClean reads one `sqlclean.toml`. Every key is optional, and without a file every default
applies.

```toml
dialect = "duckdb"            # default "generic"
include = ["**/*.sql"]
exclude = ["vendor/**"]

[[dialects]]                  # a different dialect for some paths
paths = ["warehouse/**"]
dialect = "snowflake"

[format]
line_width = 100
indent = 4

[rules]
select = ["SQLCLEAN"]              # omit to run the Rules enabled by default
ignore = ["SQLCLEAN027"]
custom = ["sql_rules"]        # Python files or directories that define custom Rules

[rules.thresholds]
max_literal_length = 100      # SQLCLEAN044
max_ranking_order_by = 5      # SQLCLEAN043

[rules.rule_options.XSQLCLEANNAME001]
prefixes = ["tmp_", "scratch_"]

[[rules.rule_exceptions]]     # one reviewed finding that is allowed
rule = "SQLCLEAN038"
path = "models/calendar.sql"
relation = "dates"
reason = "Every pair of dates is needed."

[[rules.rule_ignores]]        # Rules that do not apply to some paths
rules = ["SQLCLEAN034"]
paths = ["scratch/**"]
reason = "Exploratory queries."
```

## Finding the file

Every command uses the nearest `sqlclean.toml` in the working directory or one of its parents.
`--config <path>` uses that file instead.

The directory that holds the file is the **configuration root**. Every path in the file, every path
in findings, and the default set of files to read are relative to it. Without a file, the working
directory is the root.

## Keys

| Key | Default | Description |
|-----|---------|-------------|
| `dialect` | `"generic"` | The SQL dialect of every file without a `[[dialects]]` match |
| `include` | `["**/*.sql"]` | Glob patterns of the files to read |
| `exclude` | `[]` | Glob patterns of files never to read |
| `[[dialects]]` | none | `paths` and `dialect`: a different dialect for matching files |
| `[format] line_width` | `100` | The line width the formatter fills; see [Line width](formatting.md#line-width) |
| `[format] indent` | `4` | Spaces per indent step, from `1` to `8`; see [Indent](formatting.md#indent) |
| `[rules] select` | Rules enabled by default | Rule codes or prefixes to run |
| `[rules] ignore` | `[]` | Rule codes or prefixes to skip |
| `[rules] custom` | `[]` | Python files or directories that define [custom Rules](custom-rules/overview.md) |
| `[rules.thresholds] max_literal_length` | `100` | The longest string literal `SQLCLEAN044` allows |
| `[rules.thresholds] max_ranking_order_by` | `5` | The most `ORDER BY` expressions `SQLCLEAN043` allows in a ranking window |
| `[rules.rule_options.<CODE>]` | none | Option values of one custom Rule; see [Options](custom-rules/fixes-and-options.md#options) |
| `[[rules.rule_exceptions]]` | none | One allowed finding; see [Exceptions](rules/overview.md#exceptions) |
| `[[rules.rule_ignores]]` | none | Rules that do not apply to some paths; see [Path ignores](rules/overview.md#path-ignores) |

Selection, exceptions and ignores are described in [Rules](rules/overview.md).

## Choosing files

With no paths on the command line, a command reads every file under the configuration root whose
root-relative path matches `include` and no `exclude` pattern.

- In patterns, `*` matches within one directory and `**` matches across directories:
  `models/*.sql` reads `models/orders.sql` but not `models/marts/orders.sql`.
- In a Git repository, files that `.gitignore` ignores are skipped.
- Hidden files and directories, whose names start with `.`, are skipped.
- A directory on the command line is walked the same way.
- A file named on the command line is read even when `include` does not match it. `exclude` still
  applies.

## Dialects

`dialect` sets how every file is parsed and formatted. `[[dialects]]` entries set a different
dialect for the files their `paths` match. The first matching entry wins; files that match none use
`dialect`.

```toml
dialect = "duckdb"

[[dialects]]
paths = ["reports/**"]
dialect = "tsql"
```

A file that does not parse under its dialect is reported as
[`SQLCLEAN999`](rules/overview.md#parse-errors-sqlclean999), with the dialect in the message. Without
the `[[dialects]]` entry above, this T-SQL file under `reports/` is parsed as `duckdb`:

```sql
SELECT [order id] FROM orders
```

```text
error[SQLCLEAN999]: SQL could not be parsed as duckdb: Parse error at line 1, column 17: Expected RBracket, got Var ('id') near [SELECT [order id] FROM orders]
```

Supported dialects, case-insensitive:

| Dialect | Also accepted |
|---------|---------------|
| `generic` | |
| `athena` | |
| `bigquery` | |
| `clickhouse` | |
| `cockroachdb` | `cockroach` |
| `databricks` | |
| `datafusion` | `arrow-datafusion`, `arrow_datafusion` |
| `doris` | |
| `dremio` | |
| `drill` | |
| `druid` | |
| `duckdb` | |
| `dune` | |
| `exasol` | |
| `fabric` | |
| `hana` | `saphana`, `sap_hana` |
| `hive` | |
| `materialize` | |
| `mysql` | |
| `oracle` | |
| `postgres` | `postgresql` |
| `presto` | |
| `redshift` | |
| `risingwave` | |
| `singlestore` | `memsql` |
| `snowflake` | |
| `solr` | |
| `spark` | `spark2` |
| `sqlite` | |
| `starrocks` | |
| `tableau` | |
| `teradata` | |
| `tidb` | |
| `trino` | |
| `tsql` | `mssql`, `sqlserver` |
| `vertica` | |

## Validation

SQLClean reads and validates the file when a command starts, and exits with `2` on the first
problem it finds. `check` and `fix` also validate Rule selectors and `rule_options` before they
report any finding.

| Problem | Error |
|---------|-------|
| A key SQLClean does not know | ``unknown field `colour`, expected one of `dialect`, `include`, ...`` |
| An unknown dialect | `sqlclean.toml: unknown SQL dialect 'nosuch': ...` |
| An invalid glob pattern | `sqlclean.toml: invalid path pattern '[bad': ...` |
| `line_width` of `0` | `sqlclean.toml: [format] line_width must be positive` |
| `indent` of `0` or more than `8` | `sqlclean.toml: [format] indent must be between 1 and 8` |
| A threshold of `0` | `sqlclean.toml: [rules.thresholds] max_literal_length and max_ranking_order_by must be positive` |
| An exception or ignore without a reason | `sqlclean.toml: rule exception or ignore for SQLCLEAN001 needs a reason` |
| An exception with `cte` but no `relation` | `sqlclean.toml: rule exception for SQLCLEAN038 at models/calendar.sql sets cte without relation` |
| A `custom` path that does not exist | `[rules] custom path does not exist: sql_rules` |
| `rule_options` for a built-in Rule | `[rules.rule_options.SQLCLEAN001]: built-in Rules take no options; set limits in [rules.thresholds]` |
| `rule_options` for a code no custom Rule has | `[rules.rule_options.XSQLCLEANNOPE001]: no custom Rule has this code` |
| A selector that matches no Rule | `Rule selector 'SQLCLEAN9' matches no Rule` |

Errors are printed as `sqlclean: error: <message>`.
