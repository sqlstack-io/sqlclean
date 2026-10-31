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
    # One query with an unused CTE, a case-sensitive email comparison that a custom Rule forbids, and a NULL comparison.
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
join customers c on o.customer_id = c.customer_id
where c.email = 'Alice@example.com'
  and o.shipped_at = null
SQL
    cat > sql_rules/team.py <<'PY'
import polyglot_sql as ast  # pip install "sqlclean[ast]"

from sqlclean import Finding, RuleContext, Statement, rule

EMAIL_COLUMNS = {"email", "billing_email"}  # the columns this team treats as email addresses


@rule(
    code="XSQCTEAM001",
    message="Email comparisons must be case-insensitive",
    remediation="Compare LOWER(email) with LOWER(...) instead.",
    enabled_by_default=True,
)
def case_insensitive_email(*, statement: Statement, ctx: RuleContext) -> list[Finding]:
    starts = {
        operand.to_dict()["column"]["span"]["start"]
        for comparison in statement.polyglot_ast().find_all(ast.Eq, ast.Neq)
        for operand in (comparison.this, comparison.expression)
        if isinstance(operand, ast.Column) and operand.name.lower() in EMAIL_COLUMNS
    }
    return [
        ctx.finding(subject=statement, node=column)
        for column in statement.columns
        if column.start - statement.start in starts
    ]
PY
    ;;
  *)
    echo "unknown demo: $demo" >&2
    exit 2 ;;
esac
