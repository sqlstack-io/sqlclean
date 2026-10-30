<!-- generated-by: sqlclean skills -->

# SQLClean documentation index

Bundled copies of every page on the SQLClean documentation site, matching the installed SQLClean version. Open only the page you need.

## Getting Started

- [Introduction](index.md) (`index`) - Clean SQL, every time. Fast linting, safe fixes and one canonical format.
- [Quickstart](quickstart.md) (`quickstart`) - Install SQLClean, then check, fix and format your first SQL file.

## Concepts

- [Configuration](configuration.md) (`configuration`) - Every sqlclean.toml key, its default, per-path dialects and how the file is found.
- [Overview](rules/overview.md) (`rules/overview`) - Select Rules, record reviewed exceptions, and apply safe and unsafe fixes.
- [Rule reference](rules/reference.md) (`rules/reference`) - Every built-in Rule, whether it runs by default, and whether it has a fix.
- [Formatting](formatting.md) (`formatting`) - One canonical layout for every SQL file, the line width and indent, and files that can't be formatted.

## Custom Rules

- [Overview](custom-rules/overview.md) (`custom-rules/overview`) - Write your own Rules as Python functions over the facts of each SQL statement.
- [Statement facts](custom-rules/statement-facts.md) (`custom-rules/statement-facts`) - Every node type and field a custom Rule can read from a statement, a file and the repository.
- [Fixes and options](custom-rules/fixes-and-options.md) (`custom-rules/fixes-and-options`) - Give custom Rules safe or unsafe fixes, and typed options configured in sqlclean.toml.
- [Testing](custom-rules/testing.md) (`custom-rules/testing`) - Test custom Rules with the sqlclean.testing harness, and keep them deterministic.

## Integrations

- [Integrations](integrations.md) (`integrations`) - Run SQLClean in CI with JSON or SARIF output, in pre-commit, and from editors.
- [Embedding the engine](embedding.md) (`embedding`) - Lint, format and read statement facts from Rust, in a tool that owns its own configuration.
- [Benchmarks](benchmarks.md) (`benchmarks`) - SQLClean, sqruff and SQLFluff linting the same 10,282 Snowflake SQL files.

## CLI Reference

- [sqlclean check](cli/check.md) (`cli/check`) - Report Rule findings without changing any file.
- [sqlclean fix](cli/fix.md) (`cli/fix`) - Apply Rule fixes until the files stop changing, then format them.
- [sqlclean format](cli/format.md) (`cli/format`) - Rewrite SQL files in the canonical layout, or check that they already are.
- [sqlclean rules](cli/rules.md) (`cli/rules`) - List the built-in and custom Rules, or show one Rule's details.
- [sqlclean facts](cli/facts.md) (`cli/facts`) - Print the statement facts custom Rules read for SQL on standard input.
- [sqlclean skills](cli/skills.md) (`cli/skills`) - Install the SQLClean skill so AI coding agents know its commands, Rules and configuration.
