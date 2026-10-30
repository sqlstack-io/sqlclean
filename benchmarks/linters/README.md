# Linter benchmark

SQLClean, sqruff and SQLFluff linting the same 10,282 Snowflake SQL files. The corpus is SQLBuild's
DuckShop 10K project with the SQLBuild-specific parts removed, so every tool reads plain SQL.

## Build the corpus

The corpus comes from SQLBuild's `benchmarks/duckshop-10k` generator (generator version 2, SQLBuild
commit `def64244c`). Compiling expands its macros and drops its model headers; `convert_duckshop.py`
then replaces SQLBuild's reference functions, such as `__ref("orders")`, with plain names.

```bash
python sqlbuild/benchmarks/duckshop-10k/generate.py --out /tmp/duckshop-10k
(cd /tmp/duckshop-10k && sqb compile)
python benchmarks/linters/convert_duckshop.py /tmp/duckshop-10k/target/compiled/models /tmp/corpus
```

`/tmp/corpus/sql` then holds 10,282 `.sql` files (44 MB) and one configuration per tool, each
selecting the Snowflake dialect and nothing else, so every tool runs its default rules.

## Run

```bash
pip install sqruff sqlfluff
benchmarks/linters/bench.sh /tmp/corpus/sql /tmp/linter-bench 3
python benchmarks/linters/summarize.py /tmp/linter-bench
```

Each round copies the corpus into a fresh directory, so no tool starts with a cache, and runs every
variant in alternating order. SQLClean variants run a second time with nothing changed to measure
its result cache; sqruff and SQLFluff have no result cache. `bench.sh` records wall time, CPU time
and peak memory with `/usr/bin/time`, and every lint command writes JSON findings to stdout.

| Variant | Command |
|---|---|
| `sqlclean` | `sqlclean check --output-format json` (13 default Rules) |
| `sqlclean_all` | `sqlclean check --select SQC --output-format json` (all 45 Rules) |
| `sqlclean_custom` | `sqlclean check --config custom.toml --output-format json` (default Rules and the three Python Rules in `rules/`) |
| `sqlclean_format` | `sqlclean format --check` |
| `sqruff` | `sqruff lint -f json .` |
| `sqlfluff` | `sqlfluff lint -f json --processes $(nproc) --disable-progress-bar .` |

## Reading the results

The tools do not check the same things. Almost every sqruff and SQLFluff finding on this corpus
comes from a layout rule (`LT01`–`LT14`), which SQLClean leaves to its formatter. The closest
like-for-like comparison is therefore `sqlclean check` plus `sqlclean format --check` against the
other tools' default `lint`. Counts of findings are reported to show that every tool read the
corpus, not to compare how many problems each finds.
