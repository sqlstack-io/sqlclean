<!-- generated-by: sqlclean skills -->

# Benchmarks

> SQLClean, sqruff and SQLFluff linting the same 10,282 Snowflake SQL files.

Online: https://sqlclean.io/docs/benchmarks/

SQLClean 0.0.1, sqruff 0.44.0 and SQLFluff 4.4.0 linting the same [10,282 SQL files](#the-corpus) on
a Linux machine with 4 CPUs. Each tool runs its default rules on the Snowflake dialect, writes JSON
findings, and starts with no cache. SQLClean and sqruff cells are the median of three rounds;
SQLFluff ran once.

| Command | Wall time | CPU time | Peak memory | vs SQLClean defaults |
|---|---:|---:|---:|---:|
| `sqlclean check` (13 default Rules) | **0.81 s** | 2.5 s | 48 MiB | 1× |
| `sqlclean check --select SQLCLEAN` (all 45 Rules) | **1.33 s** | 3.8 s | 140 MiB | 1.6× |
| `sqlclean check` with three [custom Rules](#custom-rules) | **2.27 s** | 6.9 s | 182 MiB | 2.8× |
| `sqlclean format --check` | **3.03 s** | 11.5 s | 49 MiB | 3.7× |
| `sqruff lint` | 41.0 s | 114 s | 235 MiB | 51× |
| `sqlfluff lint --processes 4` | 919 s | 2,796 s | 992 MiB | 1,135× |

Run again with nothing changed, `sqlclean check` reads its result cache and finishes in 0.23 s.
sqruff and SQLFluff have no result cache, so a second run costs the same as the first.

## Custom Rules

The custom variant adds three Python Rules from `benchmarks/linters/rules/` to the default Rules:
marts must not read raw sources, staging models read only sources and seeds, and no model calls a
nondeterministic function such as `CURRENT_TIMESTAMP`. They report 3,355 findings on top of the
default Rules' 3,872.

SQLClean builds each statement's facts in Rust and evaluates statement and file Rules in one Python
process per CPU, so custom Rules use every core. A Rule over the whole repository runs in one
process, because it reads every file. Run again with nothing changed, the custom variant reads its
result cache and finishes in 0.46 s.

## Comparing like with like

The tools don't check the same things. Almost every sqruff and SQLFluff finding on this corpus
comes from a layout rule, such as indentation (`LT02`) or line length (`LT05`). SQLClean leaves
layout to its [formatter](formatting.md), so the fair comparison is `sqlclean check` plus
`sqlclean format --check`: 3.8 s together, against 41 s for sqruff and 919 s for SQLFluff.

The finding counts show that each tool read every file, not how many problems each one finds:

| Tool | Findings | Parse errors |
|---|---:|---:|
| SQLClean, default Rules | 3,872 | 0 |
| SQLClean, all Rules | 34,322 | 0 |
| sqruff | 258,378 | 0 |
| SQLFluff | 266,367 | 0 |

## The corpus

The files come from DuckShop 10K, the generated project SQLBuild benchmarks its compiler with. It
describes a fictional home goods business: storefronts, marketplaces, carriers, ledgers, ad networks
and support tools, with every name invented. Compiling the project expands its macros, and
`benchmarks/linters/convert_duckshop.py` then replaces SQLBuild's reference functions with plain
table names, so every linter reads ordinary Snowflake SQL.

| What | Count |
|---|---:|
| SQL files | 10,282 |
| Staging, intermediate and mart models | 6,149, 1,961 and 2,162 |
| Size | 44 MB |

## Run it yourself

The harness lives in `benchmarks/linters/`. Its README has the steps to build the corpus;
then:

```bash
pip install sqruff sqlfluff
benchmarks/linters/bench.sh /tmp/corpus/sql /tmp/linter-bench 3
python benchmarks/linters/summarize.py /tmp/linter-bench
```

Each round copies the corpus into a fresh directory and runs every tool in alternating order.
`/usr/bin/time` records wall time, CPU time and peak memory.
