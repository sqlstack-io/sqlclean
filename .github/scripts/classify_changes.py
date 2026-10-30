"""Decide which Verify suites a change needs from the paths it touches, one per line on stdin.

Prints `full=` and `skills=` lines for GITHUB_OUTPUT:

- Documentation, website, benchmark and image changes need only the light checks.
- Changes confined to the bundled agent skill also rebuild the executable on Linux and run the
  skills command tests, because the executable embeds the skill.
- Any other change, including workflows, Rust, Python, tests and lockfiles, runs every suite.

An empty path list means the diff could not be computed, so every suite runs.
"""

from __future__ import annotations

import fnmatch
import sys

LIGHT: tuple[str, ...] = (
    "website/*",
    "benchmarks/*",
    ".github/demos/*",
    ".github/*.png",
    ".github/*.md",
    ".github/workflows/website.yml",
    "*.md",
)
SKILL: tuple[str, ...] = ("crates/sqlclean-cli/skill/*",)


def classify(paths: list[str]) -> dict[str, bool]:
    """Return which suites the changed paths need."""

    def matches(path: str, patterns: tuple[str, ...]) -> bool:
        # `*.md` must only match a root file, while `website/*` matches every depth.
        return any(
            fnmatch.fnmatch(path, pattern) and ("/" in pattern or "/" not in path)
            for pattern in patterns
        )

    skills: bool = any(matches(path, SKILL) for path in paths)
    full: bool = not paths or any(
        not matches(path, LIGHT) and not matches(path, SKILL) for path in paths
    )
    return {"full": full, "skills": skills and not full}


def main() -> int:
    paths: list[str] = [line.strip() for line in sys.stdin if line.strip()]
    for name, value in classify(paths).items():
        print(f"{name}={str(value).lower()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
