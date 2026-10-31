"""Statement facts from the installed sqlclean executable, the same the command hosts use."""

import json
import shutil
import subprocess
import sysconfig
from pathlib import Path

from sqlclean.rule_testing.constants import EXECUTABLE_NAME, FACTS_COMMAND
from sqlclean.rule_testing.exceptions import RuleCaseError


def native_facts(*, source: str, dialect: str) -> dict[str, object]:
    """Return the native facts of source, sent as bytes so newlines stay LF on every platform."""

    completed: subprocess.CompletedProcess[bytes] = subprocess.run(
        [_executable(), FACTS_COMMAND, "--dialect", dialect],
        input=source.encode("utf-8"),
        capture_output=True,
        check=False,
    )
    if completed.returncode:
        stderr: str = completed.stderr.decode("utf-8", errors="replace").strip()
        raise RuleCaseError(f"sqlclean could not parse the case SQL: {stderr}")
    facts: dict[str, object] = json.loads(completed.stdout.decode("utf-8"))
    return facts


def _executable() -> str:
    scripts: Path = Path(sysconfig.get_path("scripts"))
    for name in (EXECUTABLE_NAME, f"{EXECUTABLE_NAME}.exe"):
        candidate: Path = scripts / name
        if candidate.is_file():
            return str(candidate)
    found: str | None = shutil.which(EXECUTABLE_NAME)
    if found is None:
        raise RuleCaseError("the sqlclean executable is not installed; install sqlclean")
    return found
