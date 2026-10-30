# Repository Agent Instructions

## Layout

- `crates/sqlclean`: the Rust engine. It holds the built-in Rules, the formatter, and the statement
  facts that custom Rules read. Other tools, such as SQLBuild, depend on this crate directly.
- `crates/sqlclean-cli`: the `sqlclean` executable. It owns configuration, file discovery, the
  result cache, fixing, reporting, and hosting Python custom Rules.
- `src/sqlclean`: the Python package. It owns the `@rule` authoring API, the custom Rule host that the
  executable starts, and the `sqlclean.testing` harness.
- `packages/sqlclean-cli`: the maturin wheel that ships the executable.

## Conventions

- Built-in Rule codes are `SQC###`. Custom Rule codes are `XSQC`, optional capital letters, and
  three digits. `SQC999` is the parse-error finding.
- Offsets in facts, findings, and edits count Unicode characters, not bytes.
- Structure follows fensu: `uv run fensu check` must report no faults for both targets. Record any
  exception in `fensu.toml` with a reason.
- Commits and pull request titles follow Conventional Commits.

## Testing

- Rebuild the executable in the environment after Rust changes: `make develop-native`.
- Run focused Python tests with xdist: `uv run pytest <paths> -q -n auto`.
- Run focused Rust tests by package: `cargo test -p <package>`.
- `make check` formats and lints everything; `make verify` is the full CI gate.

## Public Repository Hygiene

- Treat every tracked file, fixture, example, commit, branch, pull request, and release as public.
- Use neutral examples and run the public-repository hygiene scanner before the first push.
