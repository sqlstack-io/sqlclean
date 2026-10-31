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
    # One query with a NULL comparison, an unused CTE and a temporary table that a custom Rule forbids.
    mkdir -p models sql_rules
    cat > sqlclean.toml <<'TOML'
dialect = "duckdb"

[rules]
custom = ["sql_rules"]
TOML
    cat > models/orders.sql <<'SQL'
with refunds as (
    select order_id from payments where status = 'refunded'
)
select o.order_id, c.name, o.total
from orders o
join tmp_customers c on o.customer_id = c.customer_id
where o.shipped_at = null
SQL
    cat > sql_rules/naming.py <<'PY'
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
PY
    ;;
  *)
    echo "unknown demo: $demo" >&2
    exit 2 ;;
esac
