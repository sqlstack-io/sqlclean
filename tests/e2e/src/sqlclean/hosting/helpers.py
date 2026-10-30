"""Run the installed sqlclean executable on a temporary project."""

from __future__ import annotations

import os
import shutil
import subprocess
import sysconfig
from pathlib import Path


def executable() -> str:
    """Return the sqlclean executable installed beside this interpreter, else the one on PATH."""

    found: str | None = shutil.which(
        "sqlclean", path=sysconfig.get_path("scripts")
    ) or shutil.which("sqlclean")
    assert found is not None, "install sqlclean before running the command tests"
    return found


def write_project(root: Path, *, config: str, files: dict[str, str]) -> None:
    """Write sqlclean.toml and every file under root with LF line endings on every platform."""

    (root / "sqlclean.toml").write_text(config, encoding="utf-8", newline="\n")
    for path, text in files.items():
        target: Path = root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8", newline="\n")


def run(
    root: Path, *arguments: str, environment: dict[str, str]
) -> subprocess.CompletedProcess[str]:
    """Run sqlclean with arguments in root, with environment added, and return the process."""

    return subprocess.run(
        [executable(), *arguments],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
        env={**os.environ, **environment},
    )
