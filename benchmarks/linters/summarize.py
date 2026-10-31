"""Summarize bench.sh output: median wall, CPU, peak memory and findings per variant and scenario.

Usage: python summarize.py OUTDIR
"""

import json
import re
import statistics
import sys
from collections import defaultdict
from pathlib import Path

SCENARIOS: tuple[str, ...] = ("cold", "warm")
VARIANT_ORDER: tuple[str, ...] = (
    "sqlclean",
    "sqlclean_all",
    "sqlclean_custom",
    "sqlclean_format",
    "sqruff",
    "sqlfluff",
)
LABELS: dict[str, str] = {
    "sqlclean": "SQLClean, default Rules",
    "sqlclean_all": "SQLClean, every built-in Rule",
    "sqlclean_custom": "SQLClean, default Rules and three Python Rules",
    "sqlclean_format": "SQLClean formatter, `format --check`",
    "sqruff": "sqruff, default rules",
    "sqlfluff": "SQLFluff, default rules",
}
TIME_RE: re.Pattern[str] = re.compile(
    r"^(?P<variant>\w+)\.(?P<scenario>\w+)\.r(?P<round>\d+)\.time$"
)


def count_findings(variant: str, out: str) -> tuple[int, int]:
    """Return (findings, parse errors) from a tool's JSON output."""

    if variant == "sqlclean_format":
        return out.count("Would reformat: "), 0
    data = json.loads(out)
    if variant.startswith("sqlclean"):
        codes = [finding["code"] for finding in data["findings"]]
        return len(codes), codes.count("SQLCLEAN999")
    if variant == "sqruff":
        codes = [finding.get("code") for findings in data.values() for finding in findings]
        return len(codes), sum(1 for code in codes if code in (None, "PRS"))
    codes = [violation["code"] for file in data for violation in file["violations"]]
    return len(codes), sum(1 for code in codes if code in ("PRS", "TMP"))


def main() -> None:
    """Print one Markdown table row per variant and scenario."""

    out = Path(sys.argv[1])
    runs: dict[tuple[str, str], list[tuple[float, float, int, int]]] = defaultdict(list)
    findings: dict[tuple[str, str], set[tuple[int, int]]] = defaultdict(set)
    for path in sorted(out.glob("*.time")):
        match = TIME_RE.match(path.name)
        if match is None:
            continue
        stem = str(path)[: -len(".time")]
        wall, user, system, rss = path.read_text().strip().splitlines()[-1].split()
        rc = int(Path(stem + ".rc").read_text())
        key = (match["variant"], match["scenario"])
        runs[key].append((float(wall), float(user) + float(system), int(rss), rc))
        findings[key].add(count_findings(match["variant"], Path(stem + ".out").read_text()))

    print(
        "| tool | scenario | runs | wall s (median) | wall s (all) | user+sys s (median) "
        "| peak RSS MiB (median) | exit codes | findings | parse errors |"
    )
    print("|---|---|---|---|---|---|---|---|---|---|")
    found = {variant for variant, _ in runs}
    variants = [v for v in VARIANT_ORDER if v in found] + sorted(found - set(VARIANT_ORDER))
    for variant in variants:
        for scenario in SCENARIOS:
            cell = runs.get((variant, scenario))
            if not cell:
                continue
            walls = [run[0] for run in cell]
            counts = sorted(findings[(variant, scenario)])
            print(
                f"| {LABELS.get(variant, variant)} | {scenario} | {len(cell)} "
                f"| {statistics.median(walls):.2f} "
                f"| {', '.join(f'{wall:.2f}' for wall in walls)} "
                f"| {statistics.median(run[1] for run in cell):.2f} "
                f"| {statistics.median(run[2] for run in cell) / 1024:.0f} "
                f"| {','.join(sorted({str(run[3]) for run in cell}))} "
                f"| {', '.join(str(count[0]) for count in counts)} "
                f"| {', '.join(str(count[1]) for count in counts)} |"
            )


if __name__ == "__main__":
    main()
