#!/usr/bin/env bash
# Build a clean scratch project for one demo. Usage: prepare.sh <demo>
set -euo pipefail
demo="$1"
here="$(cd "$(dirname "$0")" && pwd)"
dir="$here/scratch/$demo"
rm -rf "$dir"
mkdir -p "$dir"
cd "$dir"
case "$demo" in
  check-fix)
    # One messy query: a bare JOIN, COUNT(1) and a temporary table that a custom Rule forbids.
    mkdir -p models sql_rules
    cat > sqlclean.toml <<'TOML'
dialect = "duckdb"

[rules]
select = ["SQC025", "XSQC"]
custom = ["sql_rules"]
TOML
    cat > models/orders.sql <<'SQL'
select o.order_id, c.name, count(1) as items
from orders o join tmp_customers c on o.customer_id = c.customer_id
group by o.order_id, c.name
SQL
    cat > sql_rules/style.py <<'PY'
from sqlclean import FixSafety, RuleContext, Statement, rule


@rule(
    code="XSQCNAME001",
    message="Query reads a temporary table",
    remediation="Read from the published table instead.",
    enabled_by_default=True,
)
def temporary_table(*, statement: Statement, ctx: RuleContext) -> list:
    return [
        ctx.finding(subject=statement, node=table)
        for table in statement.tables
        if table.name.startswith("tmp_")
    ]


@rule(
    code="XSQCSTYLE001",
    message="COUNT(1) should be COUNT(*)",
    remediation="Write COUNT(*).",
    fixable=FixSafety.SAFE,
    enabled_by_default=True,
)
def count_one(*, statement: Statement, ctx: RuleContext) -> list:
    counts = {f.end + 1 for f in statement.functions if f.name.upper() == "COUNT"}
    return [
        ctx.finding(subject=statement, node=literal, fix=[ctx.replace(node=literal, text="*")])
        for literal in statement.literals
        if literal.text == "1" and literal.start in counts
    ]
PY
    ;;
  *)
    echo "unknown demo: $demo" >&2
    exit 2 ;;
esac
