"""End-to-end tests for the sqlclean command on temporary projects."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

import pytest

from tests.e2e.src.sqlclean.hosting._test_types import (
    CacheTestCase,
    CheckTestCase,
    IdempotencyTestCase,
    ManyFilesTestCase,
    RewriteTestCase,
    SarifTestCase,
    SkillsTestCase,
    SummaryTestCase,
)
from tests.e2e.src.sqlclean.hosting.helpers import run, write_project
from tests.integration.src.sqlclean.rule_testing.helpers import SAMPLE_RULES

DUCKDB: str = 'dialect = "duckdb"\n'
CROSS_JOIN_ONLY: str = DUCKDB + '[rules]\nselect = ["SQC038"]\n'
CUSTOM: str = DUCKDB + '[rules]\nselect = ["XSQCNAME", "XSQCSTYLE"]\ncustom = ["sql_rules"]\n'
CROSS_JOIN_EXCEPTIONS: str = (
    CROSS_JOIN_ONLY
    + '[[rules.rule_exceptions]]\nrule = "SQC038"\npath = "models/q.sql"\nrelation = "b"\n'
    + 'reason = "Every pair is needed."\n'
    + '[[rules.rule_exceptions]]\nrule = "SQC038"\npath = "models/q.sql"\nrelation = "z"\n'
    + 'reason = "Removed join."\n'
)
NULL_AND_COMMA: str = "select a.id from a, b where a.x = null\n"


@pytest.mark.parametrize(
    "test_case",
    [
        CheckTestCase(
            description="a clean file passes",
            config=CROSS_JOIN_ONLY,
            files={"models/q.sql": "SELECT a.id FROM a\n"},
            arguments=(),
            expected_exit=0,
            expected_codes=(),
        ),
        CheckTestCase(
            description="selected Rules report in file order",
            config=DUCKDB,
            files={"models/q.sql": NULL_AND_COMMA},
            arguments=("--select", "SQC001,SQC002"),
            expected_exit=1,
            expected_codes=("SQC002", "SQC001"),
        ),
        CheckTestCase(
            description="an ignored Rule is skipped",
            config=DUCKDB,
            files={"models/q.sql": NULL_AND_COMMA},
            arguments=("--select", "SQC001,SQC002", "--ignore", "SQC002"),
            expected_exit=1,
            expected_codes=("SQC001",),
        ),
        CheckTestCase(
            description="an exception hides its finding and an unmatched one is stale",
            config=CROSS_JOIN_EXCEPTIONS,
            files={"models/q.sql": "SELECT a.id FROM a CROSS JOIN b\n"},
            arguments=(),
            expected_exit=1,
            expected_codes=(),
            expected_stale=("z",),
        ),
        CheckTestCase(
            description="a bare --config file name is read from the working directory",
            config=DUCKDB,
            files={
                "strict.toml": CROSS_JOIN_ONLY,
                "models/q.sql": "SELECT a.id FROM a CROSS JOIN b\n",
            },
            arguments=("--config", "strict.toml"),
            expected_exit=1,
            expected_codes=("SQC038",),
        ),
        CheckTestCase(
            description="lint is check by another name",
            config=CROSS_JOIN_ONLY,
            files={"models/q.sql": "SELECT a.id FROM a CROSS JOIN b\n"},
            arguments=(),
            command="lint",
            expected_exit=1,
            expected_codes=("SQC038",),
        ),
        CheckTestCase(
            description="unparseable SQL is reported as a finding",
            config=CROSS_JOIN_ONLY,
            files={"models/q.sql": "SELECT FROM WHERE (\n"},
            arguments=(),
            expected_exit=1,
            expected_codes=("SQC999",),
        ),
        CheckTestCase(
            description="custom Rules run with their configured options",
            config=CUSTOM + '[rules.rule_options.XSQCNAME001]\nprefixes = ["stg_"]\n',
            files={
                "sql_rules/naming.py": SAMPLE_RULES,
                "models/q.sql": "SELECT COUNT(1) AS n FROM stg_orders JOIN tmp_x ON TRUE\n",
            },
            arguments=(),
            expected_exit=1,
            expected_codes=("XSQCSTYLE001", "XSQCNAME001"),
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_project_when_checking_then_reports_findings_and_exit_status(
    tmp_path: Path, test_case: CheckTestCase
) -> None:
    write_project(tmp_path, config=test_case.config, files=test_case.files)

    completed: subprocess.CompletedProcess[str] = run(
        tmp_path, test_case.command, "--output-format", "json", *test_case.arguments, environment={}
    )
    report: dict[str, list[dict[str, object]]] = json.loads(completed.stdout)

    assert completed.returncode == test_case.expected_exit, completed.stderr
    assert tuple(finding["code"] for finding in report["findings"]) == test_case.expected_codes
    assert (
        tuple(stale["relation"] for stale in report["stale_exceptions"]) == test_case.expected_stale
    )


@pytest.mark.parametrize(
    "test_case",
    [
        RewriteTestCase(
            description="fix applies safe fixes, leaves unsafe ones and formats",
            config=DUCKDB + '[rules]\nselect = ["SQC001", "SQC002"]\n',
            files={"models/q.sql": NULL_AND_COMMA},
            arguments=("fix",),
            expected_exit=1,
            expected_files={
                "models/q.sql": "SELECT a.id\nFROM a\nCROSS JOIN b\nWHERE a.x = NULL\n"
            },
        ),
        RewriteTestCase(
            description="fix with unsafe fixes applies them too and then passes",
            config=DUCKDB + '[rules]\nselect = ["SQC001", "SQC002"]\n',
            files={"models/q.sql": NULL_AND_COMMA},
            arguments=("fix", "--unsafe-fixes"),
            expected_exit=0,
            expected_files={
                "models/q.sql": "SELECT a.id\nFROM a\nCROSS JOIN b\nWHERE a.x IS NULL\n"
            },
        ),
        RewriteTestCase(
            description="fix applies custom Rule fixes",
            config=CUSTOM,
            files={
                "sql_rules/naming.py": SAMPLE_RULES,
                "models/q.sql": "select count(1) as n from orders\n",
            },
            arguments=("fix",),
            expected_exit=0,
            expected_files={"models/q.sql": "SELECT COUNT(*) AS n\nFROM orders\n"},
        ),
        RewriteTestCase(
            description="fix --diff writes nothing",
            config=DUCKDB + '[rules]\nselect = ["SQC002"]\n',
            files={"models/q.sql": "select a.id from a, b\n"},
            arguments=("fix", "--diff"),
            expected_exit=1,
            expected_files={"models/q.sql": "select a.id from a, b\n"},
        ),
        RewriteTestCase(
            description="format --check fails without writing",
            config=DUCKDB,
            files={"models/q.sql": "select 1;select 2\n"},
            arguments=("format", "--check"),
            expected_exit=1,
            expected_files={"models/q.sql": "select 1;select 2\n"},
        ),
        RewriteTestCase(
            description="format writes each statement apart with one final newline",
            config=DUCKDB,
            files={"models/q.sql": "select 1;select 2"},
            arguments=("format",),
            expected_exit=0,
            expected_files={"models/q.sql": "SELECT 1;\n\nSELECT 2\n"},
        ),
        RewriteTestCase(
            description="format indents four spaces by default",
            config=DUCKDB,
            files={"models/q.sql": "with a as (select 1 as x) select x from a\n"},
            arguments=("format",),
            expected_exit=0,
            expected_files={
                "models/q.sql": "WITH a AS (\n    SELECT 1 AS x\n)\n\nSELECT x\nFROM a\n"
            },
        ),
        RewriteTestCase(
            description="format indents by the configured width",
            config=DUCKDB + "[format]\nindent = 2\n",
            files={"models/q.sql": "with a as (select 1 as x) select x from a\n"},
            arguments=("format",),
            expected_exit=0,
            expected_files={
                "models/q.sql": "WITH a AS (\n  SELECT 1 AS x\n)\n\nSELECT x\nFROM a\n"
            },
        ),
        RewriteTestCase(
            description="a file of only comments keeps its final newline",
            config=DUCKDB,
            files={"models/q.sql": "-- only a comment\n"},
            arguments=("format", "--check"),
            expected_exit=0,
            expected_files={"models/q.sql": "-- only a comment\n"},
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_project_when_rewriting_then_files_and_exit_status_match(
    tmp_path: Path, test_case: RewriteTestCase
) -> None:
    write_project(tmp_path, config=test_case.config, files=test_case.files)

    completed: subprocess.CompletedProcess[str] = run(
        tmp_path, *test_case.arguments, environment={}
    )
    files: dict[str, str] = {
        path: (tmp_path / path).read_text(encoding="utf-8") for path in test_case.expected_files
    }

    assert completed.returncode == test_case.expected_exit, completed.stdout + completed.stderr
    assert files == test_case.expected_files


@pytest.mark.parametrize(
    "test_case",
    [
        SummaryTestCase(
            description="a safe fix is fixable with plain fix",
            config=DUCKDB + '[rules]\nselect = ["SQC002"]\n',
            source="select a.id from a, b\n",
            expected_summary="Found 1 problem (1 fixable with `sqlclean fix`).",
        ),
        SummaryTestCase(
            description="an unsafe fix needs --unsafe-fixes",
            config=DUCKDB + '[rules]\nselect = ["SQC001"]\n',
            source="select a.id from a where a.x = null\n",
            expected_summary="Found 1 problem (1 fixable with `sqlclean fix --unsafe-fixes`).",
        ),
        SummaryTestCase(
            description="safe and unsafe fixes are counted apart",
            config=DUCKDB + '[rules]\nselect = ["SQC001", "SQC002"]\n',
            source=NULL_AND_COMMA,
            expected_summary=(
                "Found 2 problems (1 fixable with `sqlclean fix`, 1 more with `--unsafe-fixes`)."
            ),
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_fixable_findings_when_checking_then_summary_counts_safe_and_unsafe_fixes(
    tmp_path: Path, test_case: SummaryTestCase
) -> None:
    write_project(tmp_path, config=test_case.config, files={"q.sql": test_case.source})

    completed: subprocess.CompletedProcess[str] = run(tmp_path, "check", environment={})

    assert completed.stdout.rstrip().splitlines()[-1] == test_case.expected_summary


@pytest.mark.parametrize(
    "test_case",
    [
        SarifTestCase(
            description="the message and remediation read as two sentences",
            config=DUCKDB + '[rules]\nselect = ["SQC025"]\n',
            source="select a.id from a join b on a.id = b.id\n",
            expected_message="JOIN type is implicit. Write INNER JOIN explicitly.",
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_findings_when_reporting_sarif_then_each_result_has_a_readable_message(
    tmp_path: Path, test_case: SarifTestCase
) -> None:
    write_project(tmp_path, config=test_case.config, files={"q.sql": test_case.source})

    completed: subprocess.CompletedProcess[str] = run(
        tmp_path, "check", "--output-format", "sarif", environment={}
    )
    results: list[dict[str, dict[str, str]]] = json.loads(completed.stdout)["runs"][0]["results"]

    assert [result["message"]["text"] for result in results] == [test_case.expected_message]


@pytest.mark.parametrize(
    "test_case",
    [
        IdempotencyTestCase(
            description="a formatted query is a fixed point of the formatter",
            source="select a, b from t where a > 1 and b in (select b from u) order by a",
            expected_exits=(0, 0),
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_formatted_file_when_formatting_again_then_nothing_changes(
    tmp_path: Path, test_case: IdempotencyTestCase
) -> None:
    write_project(tmp_path, config=DUCKDB, files={"q.sql": test_case.source})

    first: subprocess.CompletedProcess[str] = run(tmp_path, "format", environment={})
    formatted: str = (tmp_path / "q.sql").read_text(encoding="utf-8")
    second: subprocess.CompletedProcess[str] = run(tmp_path, "format", "--check", environment={})

    assert (first.returncode, second.returncode) == test_case.expected_exits, second.stdout
    assert (tmp_path / "q.sql").read_text(encoding="utf-8") == formatted


@pytest.mark.parametrize(
    "test_case",
    [
        CacheTestCase(
            description="unchanged files reuse custom Rule results and edited files rerun them",
            files={
                "sql_rules/naming.py": SAMPLE_RULES,
                "models/q.sql": "SELECT * FROM tmp_orders\n",
            },
            edited={"models/q.sql": "SELECT * FROM tmp_customers\n"},
            expected_exits=(1, 1, 2),
            expected_cached_code="XSQCNAME001",
            expected_error="missing-python",
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_unchanged_files_when_checking_again_then_custom_rules_come_from_the_cache(
    tmp_path: Path, test_case: CacheTestCase
) -> None:
    write_project(tmp_path, config=CUSTOM, files=test_case.files)
    no_python: dict[str, str] = {"SQLCLEAN_PYTHON": str(tmp_path / "missing-python")}

    first: subprocess.CompletedProcess[str] = run(tmp_path, "check", environment={})
    cached: subprocess.CompletedProcess[str] = run(tmp_path, "check", environment=no_python)
    write_project(tmp_path, config=CUSTOM, files=test_case.edited)
    changed: subprocess.CompletedProcess[str] = run(tmp_path, "check", environment=no_python)

    assert (first.returncode, cached.returncode, changed.returncode) == test_case.expected_exits
    assert test_case.expected_cached_code in cached.stdout
    assert test_case.expected_error in changed.stderr


@pytest.mark.parametrize(
    "test_case",
    [
        ManyFilesTestCase(
            description="1,200 files split across hosts report every finding in file order",
            file_count=1200,
            flagged_every=3,
            expected_finding_count=400,
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_many_files_when_checking_custom_rules_then_every_host_share_reports(
    tmp_path: Path, test_case: ManyFilesTestCase
) -> None:
    flagged: range = range(0, test_case.file_count, test_case.flagged_every)
    files: dict[str, str] = {
        **{f"models/m{index:04}.sql": "SELECT a FROM x\n" for index in range(test_case.file_count)},
        **{f"models/m{index:04}.sql": "SELECT a FROM tmp_x\n" for index in flagged},
    }
    write_project(tmp_path, config=CUSTOM, files={"sql_rules/naming.py": SAMPLE_RULES, **files})

    completed: subprocess.CompletedProcess[str] = run(
        tmp_path, "check", "--output-format", "json", "--select", "XSQCNAME001", environment={}
    )
    report: dict[str, list[dict[str, object]]] = json.loads(completed.stdout)

    assert completed.returncode == 1, completed.stderr
    assert len(report["findings"]) == test_case.expected_finding_count
    assert [finding["path"] for finding in report["findings"]] == [
        f"models/m{index:04}.sql" for index in flagged
    ]


GENERATED: str = "<!-- generated-by: sqlclean skills -->"
STALE_PAGE: str = ".claude/skills/sqlclean/references/docs/removed.md"


@pytest.mark.parametrize(
    "test_case",
    [
        SkillsTestCase(
            description="default targets get the skill and its bundled docs",
            existing={},
            arguments=(),
            expected_exit=0,
            expected_entries={
                ".agents/skills/sqlclean/SKILL.md": True,
                ".agents/skills/sqlclean/references/docs/CONTENTS.md": True,
                ".claude/skills/sqlclean/references/docs/configuration.md": True,
                ".opencode/skills/sqlclean/SKILL.md": False,
            },
        ),
        SkillsTestCase(
            description="a generated page the package no longer ships is removed",
            existing={STALE_PAGE: GENERATED + "\n"},
            arguments=("--target", "claude"),
            expected_exit=0,
            expected_entries={STALE_PAGE: False, ".claude/skills/sqlclean/SKILL.md": True},
        ),
        SkillsTestCase(
            description="a hand-written skill file is not overwritten",
            existing={".claude/skills/sqlclean/SKILL.md": "my notes\n"},
            arguments=(),
            expected_exit=2,
            expected_entries={
                ".claude/skills/sqlclean/SKILL.md": False,
                ".agents/skills/sqlclean/SKILL.md": False,
            },
        ),
        SkillsTestCase(
            description="--force replaces a hand-written skill file",
            existing={".claude/skills/sqlclean/SKILL.md": "my notes\n"},
            arguments=("--force", "--target", "claude"),
            expected_exit=0,
            expected_entries={".claude/skills/sqlclean/SKILL.md": True},
        ),
    ],
    ids=lambda case: case.description,
)
def test_given_project_when_installing_skills_then_only_generated_files_are_written(
    tmp_path: Path, test_case: SkillsTestCase
) -> None:
    write_project(tmp_path, config=DUCKDB, files=test_case.existing)

    completed: subprocess.CompletedProcess[str] = run(
        tmp_path, "skills", *test_case.arguments, environment={}
    )

    assert completed.returncode == test_case.expected_exit, completed.stderr
    generated: dict[str, bool] = {
        path.relative_to(tmp_path).as_posix(): GENERATED in path.read_text(encoding="utf-8")
        for path in tmp_path.rglob("*.md")
    }
    assert {
        path: generated.get(path, False) for path in test_case.expected_entries
    } == test_case.expected_entries
