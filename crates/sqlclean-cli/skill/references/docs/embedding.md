<!-- generated-by: sqlclean skills -->

# Embedding the engine

> Lint, format and read statement facts from Rust, in a tool that owns its own configuration.

Online: https://sqlclean.io/docs/embedding/

## Contents

- Adding the dependency
- Entry points
- Lint requests
- Template code
- Fixes
- Formatting
- Statement facts
- What stays in the host

The `sqlclean` crate is the engine without the command: the built-in Rules, the formatter and the
statement facts. Tools that template, compile or generate SQL can call it in process and keep their
own configuration, file discovery and Rule codes.

```rust
use sqlclean::sql_lint::constants::LINT_API_VERSION;
use sqlclean::sql_lint::main::lint::lint;
use sqlclean::sql_lint::models::{LintRequest, LintResponse};

fn main() -> Result<(), String> {
    let request = LintRequest {
        version: LINT_API_VERSION,
        sql: "SELECT order_id FROM orders WHERE cancelled_at = NULL LIMIT 10".to_owned(),
        dialect: "duckdb".to_owned(),
        enabled_rules: Some(vec!["SQLCLEAN001".to_owned(), "SQLCLEAN004".to_owned()]),
        ..LintRequest::default()
    };
    let response: LintResponse = lint(request)?;
    for diagnostic in &response.diagnostics {
        println!("{} {}..{} {}", diagnostic.code, diagnostic.start, diagnostic.end, diagnostic.message);
        if let Some(fix) = &diagnostic.fix {
            println!("  fix: replace {}..{} with {:?}", fix.start, fix.end, fix.replacement);
        }
    }
    Ok(())
}
```

```text
SQLCLEAN001 47..48 Comparison with NULL is never true
  fix: replace 47..48 with "IS"
SQLCLEAN004 54..59 Row selection is nondeterministic
```

## Adding the dependency

The crate is not published to crates.io. Depend on it from the
[GitHub repository](https://github.com/sqlstack-io/sqlclean), pinned to a release tag or commit:

```toml
[dependencies]
sqlclean = { git = "https://github.com/sqlstack-io/sqlclean", tag = "<release tag>" }
```

## Entry points

| Function | What it does |
|----------|--------------|
| `sql_lint::main::lint::lint` | Lints one SQL text with a typed `LintRequest` |
| `sql_lint::main::engine::lint_json` | The same, with the request and response as JSON strings |
| `sql_lint::main::batch_engine::lint_batch_json` | Lints a JSON array of requests in parallel; returns an array of `{"Ok": response}` or `{"Err": message}` |
| `sql_lint::main::formatter::format_json` | Formats SQL from a JSON request |
| `sql_lint::main::catalogue::catalogue` | Every built-in Rule's code, message and remediation |
| `sql_lint::main::default_rules::default_rules` | The codes of the Rules enabled by default |
| `sql_facts::main::file_facts::file_facts` | The statement facts of a SQL text, as custom Rules read them |

`lint`, `lint_json` and `file_facts` return `Err` with a message when the SQL does not parse under
the dialect. Offsets in requests, findings, edits and facts count Unicode characters, not bytes.

## Lint requests

`version` must be `LINT_API_VERSION`; another value is rejected with
`unsupported native lint request version 2; expected 1`.

| Field | Default | Description |
|-------|---------|-------------|
| `version` | required | `LINT_API_VERSION` |
| `sql` | required | The SQL text |
| `dialect` | required | A [dialect name](configuration.md#dialects) |
| `enabled_rules` | the default Rules | Rule codes or prefixes to run |
| `ignored_rules` | `[]` | Rule codes or prefixes to skip |
| `max_literal_length` | `100` | The `SQLCLEAN044` limit |
| `max_ranking_order_by` | `5` | The `SQLCLEAN043` limit |
| `config_file` | `sqlclean.toml` | The configuration file name that remediation text refers to |
| `long_literal_guidance` | SQLClean's advice | The advice `SQLCLEAN044` gives before its `max_literal_length` setting, so a host can point at its own way to share a value |
| `externally_referenced_ctes` | `[]` | CTE names that something outside the SQL reads, so `SQLCLEAN005` does not report them |
| `relation_keys` | `{}` | Unique keys by table name as the SQL writes it, compared without case; an empty key means the relation holds at most one row, so `SQLCLEAN038` does not report a cross join with it and `SQLCLEAN018` can prove a ranking order |
| `relation_names` | `{}` | The relation each placeholder table name stands for, so `SQLCLEAN038` messages and exceptions name it |
| `external_identifiers` | `[]` | Names the host defines outside the SQL, which are not unknown qualifiers |
| `dependency_identifiers` | `[]` | Placeholder table names that stand for other relations |
| `sql_macro_outputs` | `[]` | The character range each macro call's output fills and the relations it passes, so qualification is judged as the macro wrote it |
| `allows_dynamic_output_star` | `false` | The host proved the output columns, so a top-level `SELECT *` passes `SQLCLEAN021` |
| `fixture_harness` | absent | The generated wrapper around a test or macro body; see [Template code](#template-code) |

A selector that matches no Rule is an error: `unknown native lint rule 'SQLCLEAN9'`.

## Template code

A host that lints templated SQL replaces each template expression with a placeholder before
linting, and maps offsets back afterwards. The crate exports the placeholder prefixes in
`constants`:

| Constant | Stands for |
|----------|------------|
| `INTERPOLATION_PLACEHOLDER_PREFIX` | An interpolated expression, relation or CTE list; the formatter keeps it in place |
| `MACRO_PARAMETER_PLACEHOLDER_PREFIX` | A macro parameter; `SQLCLEAN028` treats `parameter.slot` reads as written by the macro |
| `RELATION_PARAMETER_PLACEHOLDER_PREFIX` | A relation parameter, written `<prefix><index>__`; `SELECT *` from it alone passes `SQLCLEAN021` |
| `VALUE_PARAMETER_PLACEHOLDER_PREFIX` | A value parameter |

Rules never judge a placeholder as an authored name.

`fixture_harness` describes the query a test runner or macro wraps around a body. When it is
present, a terminal `SELECT 1` is ceremonial: it passes `SQLCLEAN022` and `SQLCLEAN035`, and `SQLCLEAN041` and
`SQLCLEAN042` do not apply.

| Field | Description |
|-------|-------------|
| `cte_prefixes` | Prefixes of the CTE names the harness generates, which `SQLCLEAN005` treats as reachable |
| `input_cte_prefixes` | Prefixes of the CTEs standing in for the body's inputs |
| `empty_relation` | The table function an input CTE reads with `SELECT * FROM <name>()` when it has no rows, which passes `SQLCLEAN021` |

## Fixes

`LintDiagnostic.fix` holds at most one edit: `start`, `end` and `replacement`. The engine does not
say whether a fix is safe. The `sqlclean` command treats the `SQLCLEAN001` fix as unsafe, because
turning `= NULL` into `IS NULL` changes which rows match, and every other built-in fix as safe. A
host that applies fixes makes the same decision. See
[Safe and unsafe fixes](rules/overview.md#safe-and-unsafe-fixes).

## Formatting

```rust
let request = r#"{"version": 1, "sql": "select order_id, amount from orders where amount > 0", "dialect": "duckdb", "line_width": 100}"#;
println!("{}", format_json(request)?);
```

```json
{"version":1,"sql":"SELECT\n  order_id,\n  amount\nFROM orders\nWHERE amount > 0","changed":true,"formatted":true}
```

`indent` sets the spaces per indent step. Without it the engine indents two spaces, its layout
before the `sqlclean` command chose four; pass `"indent": 4` to match `sqlclean format`.

When the formatter cannot format the SQL, `formatted` is `false`, `sql` is the input unchanged,
and `reason` says why.

`format_json` formats the text it is given as one unit. `sqlclean format` formats each statement on
its own, joins them with one blank line and ends the file with a newline. To produce the same
layout, split the file with `file_facts`, whose statements carry `start` and `terminated_end`, and
format each statement. See [Formatting](formatting.md#statements).

## Statement facts

```rust
let facts = file_facts("SELECT o.order_id FROM orders AS o", "duckdb")?;
for statement in &facts.statements {
    for table in &statement.tables {
        println!("{} {} {}..{}", statement.kind, table.qualified_name, table.start, table.end);
    }
}
```

```text
select orders 23..29
```

The fields are those of [Statement facts](custom-rules/statement-facts.md), without `line`,
`column` and `text`. Functions are `NamedNode` values with a `name`.

## What stays in the host

The crate reads no files, no configuration and no cache, and does not run Python. A host owns:

- which files to lint and in which dialect;
- Rule selection, which it passes as `enabled_rules` and `ignored_rules`;
- exceptions, ignores and their reasons;
- the codes and configuration it shows its users;
- caching, and applying fixes.

The [`sqlclean` command](cli/check.md) is one such host. Its behaviour, described in the rest of
these docs, is a reference for what each of these pieces does.
