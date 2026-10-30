<!-- generated-by: sqlclean skills -->

# sqlclean format

> Rewrite SQL files in the canonical layout, or check that they already are.

Online: https://sqlclean.io/docs/cli/format/

`sqlclean format` rewrites SQL files in SQLClean's canonical layout. It runs no Rules and applies
no fixes; use [`sqlclean fix`](fix.md) for both.

## Usage

```bash
sqlclean format [OPTIONS] [PATHS]...
```

```bash
sqlclean format                    # rewrite every matching file
sqlclean format --check            # fail when a file needs formatting
sqlclean format --diff reports/    # print the changes for one directory
```

Files are chosen as for [`sqlclean check`](check.md). Each file is formatted with its own
[dialect](../configuration.md#dialects) and the configured
[`line_width`](../formatting.md#line-width) and [`indent`](../formatting.md#indent).

## Flags

| Flag | Description |
|------|-------------|
| `PATHS...` | Files or directories to format; the configuration root when omitted |
| `--check` | Fail when a file needs formatting, without writing it |
| `--diff` | Print the changes as a unified diff instead of writing them |
| `--config <PATH>` | Use this configuration file instead of the nearest `sqlclean.toml` |

## Output

```bash
sqlclean format --check
```

```text
j.sql: could not format: Parse error at line 1, column 16: Expected Colon, got RBrace ('}') near [{{ col }} from orders]
Would reformat: models_a.sql
1 file(s) would be reformatted, 2 file(s) already formatted.
```

`--check` names each file that would change. `--diff` prints a unified diff instead:

```diff
--- a/models_a.sql
+++ b/models_a.sql
@@ -1 +1,5 @@
-select order_id, amount from orders where amount > 0
+SELECT
+    order_id,
+    amount
+FROM orders
+WHERE amount > 0
```

Without either flag, files are rewritten in place and the summary reads
`N file(s) reformatted, N file(s) already formatted.` The summary and the `could not format`
lines go to stderr; the `Would reformat` lines and diffs go to stdout.

A file the formatter cannot format, usually because it does not parse under its dialect, is left
unchanged and reported with the reason. See
[When a file can't be formatted](../formatting.md#when-a-file-cant-be-formatted).

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Every file is formatted, or was rewritten |
| `1` | A file could not be formatted, or `--check` or `--diff` found a file to change |
| `2` | The command could not run: invalid arguments, configuration or input |
