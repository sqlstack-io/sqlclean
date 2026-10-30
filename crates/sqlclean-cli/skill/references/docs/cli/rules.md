<!-- generated-by: sqlclean skills -->

# sqlclean rules

> List the built-in and custom Rules, or show one Rule's details.

Online: https://sqlclean.io/docs/cli/rules/

## Contents

- Usage
- Flags
- Listing Rules
- Showing one Rule
- JSON output
- Exit codes

`sqlclean rules` lists every Rule the configuration can select: the built-in Rules, then the custom
Rules under `[rules] custom`. With a code, it shows that Rule.

## Usage

```bash
sqlclean rules [OPTIONS] [CODE]
```

```bash
sqlclean rules                 # list every Rule
sqlclean rules SQC001          # show one Rule
sqlclean rules --json          # every Rule as JSON
sqlclean rules --json SQC001   # one Rule as JSON
```

## Flags

| Flag | Description |
|------|-------------|
| `CODE` | Show this Rule's details; an exact code, not a prefix |
| `--json` | Print Rules as a JSON array |
| `--config <PATH>` | Use this configuration file instead of the nearest `sqlclean.toml` |

## Listing Rules

```text
SQC001       default  Comparison with NULL is never true
SQC002       default  Comma-separated sources create an implicit cartesian join
SQC003       default  Non-cross join has no meaningful condition
SQC004       default  Row selection is nondeterministic
...
SQC045                JSON payload is parsed more than once in one SELECT
XSQCNAME001  default  Query reads a temporary table
XSQCSTYLE001 default  COUNT(1) should be COUNT(*)
```

`default` marks the Rules that run when `[rules] select` is not set. See
[Default Rules](../rules/overview.md#default-rules) and the
[Rule reference](../rules/reference.md).

## Showing one Rule

```bash
sqlclean rules SQC001
```

```text
SQC001: Comparison with NULL is never true
Subject: statement
Enabled by default: yes
Fixable: unsafe
Remediation: Use IS NULL or IS NOT NULL when testing for NULL.
```

A custom Rule also shows the Python module that defines it:

```text
XSQCSTYLE001: COUNT(1) should be COUNT(*)
Subject: statement
Enabled by default: yes
Defined in: sql_rules.naming
Fixable: safe
Remediation: Write COUNT(*).
```

`Fixable` is left out when the Rule has no fix.

## JSON output

```bash
sqlclean rules --json XSQCNAME001
```

```json
[
  {
    "code": "XSQCNAME001",
    "message": "Query reads a temporary table",
    "remediation": "Read from the published table instead.",
    "enabled_by_default": true,
    "custom": true,
    "subject": "statement",
    "module": "sql_rules.naming"
  }
]
```

| Field | Description |
|-------|-------------|
| `code` | The Rule code |
| `message` | The finding message |
| `remediation` | What to do about a finding |
| `enabled_by_default` | Whether the Rule runs when `[rules] select` is not set |
| `custom` | `true` for custom Rules |
| `subject` | `statement`, `file` or `repository` |
| `fixable` | `safe` or `unsafe`; absent when the Rule has no fix |
| `module` | The Python module of a custom Rule; absent for built-in Rules |

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | The Rules were printed |
| `2` | Unknown Rule code, or the configuration or a custom Rule module could not be loaded |
