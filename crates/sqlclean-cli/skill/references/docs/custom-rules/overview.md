<!-- generated-by: sqlclean skills -->

# Overview

> Write your own Rules as Python functions over the facts of each SQL statement.

Online: https://sqlclean.io/docs/custom-rules/overview/

## Contents

- Where Rules live
- Declaring a Rule
- Choosing a subject
- Reporting findings
- How Rules run
- Python environment
- Errors
- Continue

A custom Rule is a Python function decorated with `@rule`. It receives a statement, a file or the
whole repository, and returns findings. Custom Rules use the same selection, exceptions, fixes,
output and cache as the built-in Rules.

```python
# sql_rules/naming.py
from sqlclean import Finding, RuleContext, Statement, rule

@rule(
    code="XSQCNAME001",
    message="Query reads a temporary table",
    remediation="Read from the published table instead.",
    enabled_by_default=True,
)
def temporary_table(*, statement: Statement, ctx: RuleContext) -> list[Finding]:
    return [
        ctx.finding(subject=statement, node=table)
        for table in statement.tables
        if table.name.startswith("tmp_")
    ]
```

Point `[rules] custom` at the file or its directory:

```toml
[rules]
custom = ["sql_rules"]
```

```text
error[XSQCNAME001]: Query reads a temporary table
  --> models/orders.sql:2:6
   |
 2 | FROM tmp_orders
   |      ^^^^^^^^^^
  = help: Read from the published table instead.
```

## Where Rules live

`[rules] custom` lists Python files or directories, relative to the configuration root. SQLClean
imports every `.py` file under them and registers each function decorated with `@rule`. Other
functions, constants and classes stay ordinary Python.

- Files named `test_*.py`, `conftest.py` and `__init__.py` are not loaded as Rules, so tests can
  live next to the Rules they test.
- Modules are imported by their path from the configuration root: `sql_rules/naming.py` is
  `sql_rules.naming`. A Rule module can import a helper the same way, for example
  `from sql_rules.helpers import table_prefixes`.
- Two Rules with the same code are an error: `custom Rule code XSQCNAME001 is defined twice`.

## Declaring a Rule

| `@rule` argument | Default | Description |
|------------------|---------|-------------|
| `code` | required | `XSQC`, optional capital letters, and three digits, such as `XSQCNAME001` |
| `message` | required | The finding message |
| `remediation` | required | What to do about a finding |
| `enabled_by_default` | `False` | Run the Rule when `[rules] select` is not set |
| `fixable` | `None` | `FixSafety.SAFE` or `FixSafety.UNSAFE` when findings carry fixes |
| `options` | `()` | Typed `RuleOption`s configured in `sqlclean.toml` |
| `slug` | the code in lower case | A short name for the Rule |

A Rule that is not enabled by default runs when `select` names it, by code or by a prefix such as
`XSQC` or `XSQCNAME`. See [Selecting Rules](../rules/overview.md#selecting-rules). Fixes and
options are covered in [Fixes and options](fixes-and-options.md).

## Choosing a subject

The function takes exactly two keyword-only parameters: one subject and one `RuleContext`. Their
type annotations decide how the Rule is called; their names and order do not matter.

| Subject | Called | Use it for |
|---------|--------|------------|
| `statement: Statement` | Once per statement | Checks of one query, such as the tables it reads |
| `file: SqlFile` | Once per file | Checks of a file as a whole, such as how many statements it holds |
| `repository: Repository` | Once over every checked file | Conventions across files |

```python
from collections import Counter

from sqlclean import Finding, Repository, RuleContext, SqlFile, rule

@rule(
    code="XSQCFILE001",
    message="File holds more than one statement",
    remediation="Split the file so each holds one statement.",
)
def one_statement(*, file: SqlFile, ctx: RuleContext) -> list[Finding]:
    return [ctx.finding(subject=file)] if len(file.statements) > 1 else []

@rule(
    code="XSQCREPO001",
    message="Two files define a CTE with the same name",
    remediation="Give each CTE a name that is unique across files.",
)
def duplicate_cte_names(*, repository: Repository, ctx: RuleContext) -> list[Finding]:
    statements = [s for file in repository.files for s in file.statements]
    counts = Counter(cte.name for s in statements for cte in s.ctes)
    return [
        ctx.finding(subject=s, node=cte)
        for s in statements
        for cte in s.ctes
        if counts[cte.name] > 1
    ]
```

A Repository Rule is evaluated again whenever any checked file changes, so prefer a statement or
file subject when the check needs only one file.

[Statement facts](statement-facts.md) lists everything a subject exposes.

## Reporting findings

A Rule returns a list of findings made with `ctx.finding`:

| Argument | Description |
|----------|-------------|
| `subject` | The `Statement` or `SqlFile` the finding is in |
| `node` | Where to report it: any node or statement; the start of `subject` when omitted |
| `message` | Replaces the Rule's message for this finding |
| `remediation` | Replaces the Rule's remediation for this finding |
| `fix` | A list of edits; see [Fixes](fixes-and-options.md#fixes) |

A Repository Rule passes the statement or file each finding belongs to as `subject`.

## How Rules run

- Custom Rules run after the built-in Rules, on files that parse. A file with a
  [parse error](../rules/overview.md#parse-errors-sqc999) gets no custom Rule findings.
- From 500 files, Statement and File Rules split the files across one Python process per CPU. A
  Rule can't share state between files, so anything that spans files belongs in a Repository Rule,
  which always runs in one process with every file.
- The working directory is the configuration root, and `PYTHONHASHSEED` is `0`, so iterating a set
  of strings gives the same order on every run.
- Results are cached with the built-in results. Editing any Python file under `[rules] custom`
  evaluates the custom Rules again. See [Caching](../rules/overview.md#caching).
- Don't print to standard output from a Rule. The command reads Rule results from it and fails
  with `custom Rules host returned no result`.

## Python environment

Custom Rules run in a Python interpreter that has the `sqlclean` package installed. SQLClean uses,
in order:

1. the interpreter named by the `SQLCLEAN_PYTHON` environment variable;
2. the Python installed next to the `sqlclean` executable, such as the one in the same virtual
   environment;
3. `python3` on the `PATH`.

When none has the package, the command fails with:

```text
Custom Rules need the sqlclean Python package in the environment that runs sqlclean: install it with `pip install sqlclean`, or set SQLCLEAN_PYTHON to an interpreter that has it.
```

## Errors

A Rule that cannot load or run stops the command with exit code `2`:

| Problem | Error |
|---------|-------|
| An invalid code | `custom Rule code must be XSQC, optional capital letters and 3 digits, such as XSQCNAME001: XSQC01` |
| A wrong signature | `custom Rule XSQCBAD001 must accept exactly one typed Statement, SqlFile or Repository subject and one typed RuleContext, both keyword-only` |
| A module that fails to import | `could not import custom Rule module sql_rules.bad: <error>` |
| An exception in the Rule | `custom Rule XSQCBAD001 failed on models/a.sql: IndexError: tuple index out of range`, then the traceback |
| A return value that is not a list of findings | `custom Rule XSQCRET001 must return a list of findings from ctx.finding(...)` |
| A fix from a Rule without `fixable` | `Rule XSQCBAD001 returned a fix but does not declare fixable` |

## Continue

    Every node type and field a Rule can read.
    Add safe or unsafe fixes, and typed options set in `sqlclean.toml`.
    Test Rules with the `sqlclean.testing` harness.
    See the facts of any SQL from the command line.
