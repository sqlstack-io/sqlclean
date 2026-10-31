#!/usr/bin/env bash
# Linter benchmark: SQLClean vs sqruff vs SQLFluff on the plain-SQL DuckShop 10K corpus.
#
# Usage: bench.sh CORPUS OUTDIR ROUNDS [VARIANT...]
#
# CORPUS is OUTDIR/sql from convert_duckshop.py: the SQL files plus a configuration for each linter
# that selects the Snowflake dialect. Each round runs every variant once, in alternating order (odd
# rounds forward, even rounds reversed). For each variant and round:
#   1. copy the pristine corpus into a work directory (no caches)
#   2. cold: lint every file
#   3. warm: lint again with nothing changed (only for variants listed in WARM_VARIANTS)
#
# Every tool writes JSON findings to stdout, so each run also pays for its own reporting.
#
# Environment:
#   SQLCLEAN  sqlclean executable (default: sqlclean)
#   SQRUFF    sqruff executable (default: sqruff)
#   SQLFLUFF  sqlfluff executable (default: sqlfluff)
#   PROCESSES SQLFluff worker processes (default: nproc). SQLClean and sqruff pick their own.
#
# Default variants: sqlclean sqlclean_all sqlclean_custom sqlclean_format sqruff sqlfluff.
set -u
CORPUS=$(realpath "$1")
OUT=$(realpath -m "$2")
ROUNDS=$3
shift 3
HERE=$(cd "$(dirname "$0")" && pwd)
SQLCLEAN=${SQLCLEAN:-sqlclean}
SQRUFF=${SQRUFF:-sqruff}
SQLFLUFF=${SQLFLUFF:-sqlfluff}
PROCESSES=${PROCESSES:-$(nproc)}
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

declare -A CMD
CMD[sqlclean]="$SQLCLEAN check --output-format json"
CMD[sqlclean_all]="$SQLCLEAN check --select SQLCLEAN --output-format json"
# The default Rules plus the three Python Rules in rules/, copied into the work directory.
CMD[sqlclean_custom]="$SQLCLEAN check --config custom.toml --output-format json"
# sqruff's and SQLFluff's default rules include layout; SQLClean checks layout with its formatter.
CMD[sqlclean_format]="$SQLCLEAN format --check"
CMD[sqruff]="$SQRUFF lint -f json ."
CMD[sqlfluff]="$SQLFLUFF lint -f json --processes $PROCESSES --disable-progress-bar ."
# SQLClean reuses results for unchanged files; the others have no result cache.
WARM_VARIANTS=" sqlclean sqlclean_all sqlclean_custom "

VARIANTS=("$@")
if [ ${#VARIANTS[@]} -eq 0 ]; then
  VARIANTS=(sqlclean sqlclean_all sqlclean_custom sqlclean_format sqruff sqlfluff)
fi
for v in "${VARIANTS[@]}"; do
  if [ -z "${CMD[$v]+set}" ]; then
    echo "unknown variant: $v" >&2
    exit 2
  fi
done
mkdir -p "$OUT"
{
  echo "files $(find "$CORPUS" -name '*.sql' | wc -l)"
  echo "nproc $(nproc)"
  echo "processes $PROCESSES"
  "$SQLCLEAN" --version
  "$SQRUFF" --version
  "$SQLFLUFF" --version
} >"$OUT/environment.txt" 2>&1

load1() { cut -d' ' -f1 /proc/loadavg; }

measure() {
  local v=$1 scenario=$2 round=$3 base before after
  base="$OUT/$v.$scenario.r$round"
  echo "${CMD[$v]}" >"$base.cmd"
  before=$(load1)
  /usr/bin/time -f '%e %U %S %M' -o "$base.time" \
    bash -c "cd '$WORK/$v' && exec ${CMD[$v]}" >"$base.out" 2>"$base.err"
  echo $? >"$base.rc"
  after=$(load1)
  echo "$v $scenario r$round load $before $after" >>"$OUT/uptime.log"
  echo "$v $scenario r$round rc=$(cat "$base.rc") time=$(tail -1 "$base.time") load=$before/$after" |
    tee -a "$OUT/summary.log"
}

for round in $(seq 1 "$ROUNDS"); do
  n=${#VARIANTS[@]}
  for i in $(seq 0 $((n - 1))); do
    if [ $((round % 2)) -eq 0 ]; then v=${VARIANTS[$((n - 1 - i))]}; else v=${VARIANTS[$i]}; fi
    rm -rf "${WORK:?}/$v"
    cp -a "$CORPUS" "$WORK/$v"
    if [ "$v" = sqlclean_custom ]; then
      cp -r "$HERE/rules" "$WORK/$v/bench_rules"
      printf 'dialect = "snowflake"\n[rules]\ncustom = ["bench_rules"]\n' >"$WORK/$v/custom.toml"
    fi
    sync
    measure "$v" cold "$round"
    [[ $WARM_VARIANTS == *" $v "* ]] && measure "$v" warm "$round"
    rm -rf "${WORK:?}/$v"
  done
done
