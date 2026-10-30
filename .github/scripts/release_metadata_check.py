"""Reject release branches containing non-generated metadata changes."""

from __future__ import annotations

import argparse
import copy
import json
import re
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

SEMVER_PART_COUNT: int = 3
NORMALIZED_VERSION: str = "0.0.0"
CHANGELOG_PATH: str = "CHANGELOG.md"
RELEASE_MANIFEST_PATH: str = ".release-please-manifest.json"
ROOT_PROJECT_PATH: str = "pyproject.toml"
CLI_PROJECT_PATH: str = "packages/sqlclean-cli/pyproject.toml"
CARGO_WORKSPACE_PATH: str = "Cargo.toml"
CARGO_LOCK_PATH: str = "Cargo.lock"
UV_LOCK_PATH: str = "uv.lock"
CLI_DISTRIBUTION: str = "sqlclean-cli"
VERSION_PATHS: tuple[str, ...] = (
    RELEASE_MANIFEST_PATH,
    ROOT_PROJECT_PATH,
    CLI_PROJECT_PATH,
    CARGO_WORKSPACE_PATH,
    UV_LOCK_PATH,
    CARGO_LOCK_PATH,
)
# Files that may not exist before the first release.
OPTIONAL_BASE_PATHS: frozenset[str] = frozenset({CHANGELOG_PATH})
OWNED_CARGO_PACKAGES: frozenset[str] = frozenset({"sqlclean", "sqlclean-cli"})
OWNED_UV_PACKAGES: frozenset[str] = frozenset({"sqlclean", "sqlclean-cli"})
ALLOWED_PATHS: frozenset[str] = frozenset((*VERSION_PATHS, CHANGELOG_PATH))
# The TOML table holding the owned version field in each manifest.
MANIFEST_SECTIONS: dict[str, tuple[str, ...]] = {
    ROOT_PROJECT_PATH: ("project",),
    CLI_PROJECT_PATH: ("project",),
    CARGO_WORKSPACE_PATH: ("workspace", "package"),
}


@dataclass(frozen=True)
class ReleaseDelta:
    """Base and release-head content for generated metadata files."""

    old_version: str
    new_version: str
    changed_paths: frozenset[str]
    base_files: dict[str, str]
    head_files: dict[str, str]


def load_release_delta(*, repo_root: Path, base_ref: str, head_ref: str) -> ReleaseDelta:
    """Read the changed metadata and package versions from two Git revisions."""

    changed: list[str] = _git(
        "diff", "--name-only", base_ref, head_ref, repo_root=repo_root
    ).splitlines()
    base_files: dict[str, str] = {
        path: _show(ref=base_ref, path=path, repo_root=repo_root) for path in ALLOWED_PATHS
    }
    head_files: dict[str, str] = {
        path: _git("show", f"{head_ref}:{path}", repo_root=repo_root) for path in ALLOWED_PATHS
    }
    old_version: str = tomllib.loads(base_files[ROOT_PROJECT_PATH])["project"]["version"]
    new_version: str = tomllib.loads(head_files[ROOT_PROJECT_PATH])["project"]["version"]
    return ReleaseDelta(
        old_version=old_version,
        new_version=new_version,
        changed_paths=frozenset(changed),
        base_files=base_files,
        head_files=head_files,
    )


def get_release_delta_errors(
    *,
    delta: ReleaseDelta,
    allow_stale_lockfiles: bool = False,
) -> list[str]:
    """Return every unsafe or inconsistent release delta."""

    errors: list[str] = []
    unexpected: list[str] = sorted(delta.changed_paths - ALLOWED_PATHS)
    if unexpected:
        errors.append(f"unexpected release files: {', '.join(unexpected)}")
    if not is_forward_semver(old_version=delta.old_version, new_version=delta.new_version):
        errors.append("release version must be a forward MAJOR.MINOR.PATCH transition")
    for path in VERSION_PATHS:
        if not version_file_is_valid(
            path=path,
            delta=delta,
            allow_stale_lockfiles=allow_stale_lockfiles,
        ):
            errors.append(f"{path} contains changes beyond the release version substitution")
    changelog: str = delta.head_files[CHANGELOG_PATH]
    base_history: str = delta.base_files[CHANGELOG_PATH].partition("\n\n")[2]
    if not changelog.startswith("# Changelog\n\n") or base_history not in changelog:
        errors.append("CHANGELOG.md rewrites existing release history")
    if f"## [{delta.new_version}]" not in changelog:
        errors.append(f"CHANGELOG.md has no {delta.new_version} release heading")
    return errors


def is_forward_semver(*, old_version: str, new_version: str) -> bool:
    """Return whether two plain semantic versions form a forward transition."""

    old_semver: tuple[int, int, int] | None = _parse_semver(value=old_version)
    new_semver: tuple[int, int, int] | None = _parse_semver(value=new_version)
    return old_semver is not None and new_semver is not None and new_semver > old_semver


def version_file_is_valid(
    *,
    path: str,
    delta: ReleaseDelta,
    allow_stale_lockfiles: bool,
) -> bool:
    """Compare one parsed metadata file after normalizing owned version fields."""

    try:
        semantic_valid: bool
        if path == RELEASE_MANIFEST_PATH:
            semantic_valid = _release_manifest_is_valid(delta=delta, path=path)
        else:
            base: dict[str, Any] = tomllib.loads(delta.base_files[path])
            head: dict[str, Any] = tomllib.loads(delta.head_files[path])
            if path == ROOT_PROJECT_PATH:
                semantic_valid = _root_project_is_valid(base=base, head=head, delta=delta)
            elif path in MANIFEST_SECTIONS:
                semantic_valid = _manifest_is_valid(
                    base=base,
                    head=head,
                    section=MANIFEST_SECTIONS[path],
                    delta=delta,
                )
            elif path == CARGO_LOCK_PATH:
                semantic_valid = _lock_is_valid(
                    base=base,
                    head=head,
                    owned_packages=OWNED_CARGO_PACKAGES,
                    delta=delta,
                    allow_old=allow_stale_lockfiles,
                )
            elif path == UV_LOCK_PATH:
                semantic_valid = _lock_is_valid(
                    base=base,
                    head=head,
                    owned_packages=OWNED_UV_PACKAGES,
                    delta=delta,
                    allow_old=allow_stale_lockfiles,
                )
            else:
                return False
        return semantic_valid and _version_text_is_valid(
            path=path,
            delta=delta,
            allow_stale_lockfiles=allow_stale_lockfiles,
        )
    except (KeyError, TypeError, ValueError, tomllib.TOMLDecodeError):
        return False


def _parse_semver(*, value: str) -> tuple[int, int, int] | None:
    parts: list[str] = value.split(".")
    if len(parts) != SEMVER_PART_COUNT or any(not part.isdigit() for part in parts):
        return None
    return (int(parts[0]), int(parts[1]), int(parts[2]))


def _release_manifest_is_valid(*, delta: ReleaseDelta, path: str) -> bool:
    base: dict[str, Any] = json.loads(delta.base_files[path])
    head: dict[str, Any] = json.loads(delta.head_files[path])
    base_normalized: dict[str, Any] | None = _normalized_field(
        document=base,
        section=(),
        field=".",
        expected=delta.old_version,
    )
    head_normalized: dict[str, Any] | None = _normalized_field(
        document=head,
        section=(),
        field=".",
        expected=delta.new_version,
    )
    return base_normalized is not None and base_normalized == head_normalized


def _manifest_is_valid(
    *,
    base: dict[str, Any],
    head: dict[str, Any],
    section: tuple[str, ...],
    delta: ReleaseDelta,
) -> bool:
    base_normalized: dict[str, Any] | None = _normalized_field(
        document=base,
        section=section,
        field="version",
        expected=delta.old_version,
    )
    head_normalized: dict[str, Any] | None = _normalized_field(
        document=head,
        section=section,
        field="version",
        expected=delta.new_version,
    )
    return base_normalized is not None and base_normalized == head_normalized


def _root_project_is_valid(
    *,
    base: dict[str, Any],
    head: dict[str, Any],
    delta: ReleaseDelta,
) -> bool:
    base_normalized: dict[str, Any] | None = _normalized_field(
        document=base,
        section=("project",),
        field="version",
        expected=delta.old_version,
    )
    head_normalized: dict[str, Any] | None = _normalized_field(
        document=head,
        section=("project",),
        field="version",
        expected=delta.new_version,
    )
    if base_normalized is None or head_normalized is None:
        return False
    base_normalized = _normalized_cli_dependency(
        document=base_normalized,
        version=delta.old_version,
    )
    head_normalized = _normalized_cli_dependency(
        document=head_normalized,
        version=delta.new_version,
    )
    return base_normalized is not None and base_normalized == head_normalized


def _normalized_field(
    *,
    document: dict[str, Any],
    section: tuple[str, ...],
    field: str,
    expected: str,
) -> dict[str, Any] | None:
    normalized: dict[str, Any] = copy.deepcopy(document)
    target: dict[str, Any] = normalized
    for key in section:
        target = target[key]
    if target.get(field) != expected:
        return None
    target[field] = NORMALIZED_VERSION
    return normalized


def _normalized_cli_dependency(
    *,
    document: dict[str, Any],
    version: str,
) -> dict[str, Any] | None:
    requirement: str = f"{CLI_DISTRIBUTION}=={version}"
    dependencies: list[Any] = document["project"].get("dependencies", [])
    if dependencies.count(requirement) != 1:
        return None
    normalized: dict[str, Any] = copy.deepcopy(document)
    normalized["project"]["dependencies"] = [
        f"{CLI_DISTRIBUTION}=={NORMALIZED_VERSION}" if item == requirement else item
        for item in dependencies
    ]
    return normalized


def _lock_is_valid(
    *,
    base: dict[str, Any],
    head: dict[str, Any],
    owned_packages: frozenset[str],
    delta: ReleaseDelta,
    allow_old: bool,
) -> bool:
    base_normalized: dict[str, Any] | None = _normalized_lock(
        document=base,
        owned_packages=owned_packages,
        allowed_versions=frozenset({delta.old_version}),
    )
    allowed_head: frozenset[str] = frozenset(
        {delta.old_version, delta.new_version} if allow_old else {delta.new_version}
    )
    head_normalized: dict[str, Any] | None = _normalized_lock(
        document=head,
        owned_packages=owned_packages,
        allowed_versions=allowed_head,
    )
    return base_normalized is not None and base_normalized == head_normalized


def _normalized_lock(
    *,
    document: dict[str, Any],
    owned_packages: frozenset[str],
    allowed_versions: frozenset[str],
) -> dict[str, Any] | None:
    normalized: dict[str, Any] = copy.deepcopy(document)
    found: set[str] = set()
    for package in normalized.get("package", []):
        if package.get("name") not in owned_packages:
            continue
        found.add(package["name"])
        if package.get("version") not in allowed_versions:
            return None
        package["version"] = NORMALIZED_VERSION
    return normalized if found == owned_packages else None


def _version_text_is_valid(
    *,
    path: str,
    delta: ReleaseDelta,
    allow_stale_lockfiles: bool,
) -> bool:
    base: str = delta.base_files[path]
    expected: str | None
    if path == RELEASE_MANIFEST_PATH:
        expected = _replace_once(
            value=base,
            pattern=rf'("\."\s*:\s*"){re.escape(delta.old_version)}(")',
            replacement=rf"\g<1>{delta.new_version}\g<2>",
        )
    elif path in MANIFEST_SECTIONS:
        expected = _replace_manifest_version(
            value=base,
            section=".".join(MANIFEST_SECTIONS[path]),
            old_version=delta.old_version,
            new_version=delta.new_version,
        )
        if expected is not None and path == ROOT_PROJECT_PATH:
            expected = _replace_once(
                value=expected,
                pattern=re.escape(f"{CLI_DISTRIBUTION}=={delta.old_version}"),
                replacement=f"{CLI_DISTRIBUTION}=={delta.new_version}",
            )
    elif path == CARGO_LOCK_PATH:
        expected = _replace_lock_versions(
            value=base,
            owned_packages=OWNED_CARGO_PACKAGES,
            old_version=delta.old_version,
            new_version=delta.new_version,
        )
    elif path == UV_LOCK_PATH:
        expected = _replace_lock_versions(
            value=base,
            owned_packages=OWNED_UV_PACKAGES,
            old_version=delta.old_version,
            new_version=delta.new_version,
        )
    else:
        return False
    if expected is None:
        return False
    head: str = delta.head_files[path]
    return head == expected or (
        allow_stale_lockfiles and path in {CARGO_LOCK_PATH, UV_LOCK_PATH} and head == base
    )


def _replace_manifest_version(
    *,
    value: str,
    section: str,
    old_version: str,
    new_version: str,
) -> str | None:
    return _replace_once(
        value=value,
        pattern=(
            rf'(?ms)(^\[{re.escape(section)}\]\s*$.*?^version\s*=\s*")'
            rf"{re.escape(old_version)}"
            r'("[^\n]*$)'
        ),
        replacement=rf"\g<1>{new_version}\g<2>",
    )


def _replace_lock_versions(
    *,
    value: str,
    owned_packages: frozenset[str],
    old_version: str,
    new_version: str,
) -> str | None:
    blocks: list[str] = re.split(r"(?=^\[\[package\]\]\s*$)", value, flags=re.MULTILINE)
    found: set[str] = set()
    for index, block in enumerate(blocks):
        name_match: re.Match[str] | None = re.search(r'^name\s*=\s*"([^"]+)"', block, re.MULTILINE)
        if name_match is None or name_match.group(1) not in owned_packages:
            continue
        found.add(name_match.group(1))
        replaced: str | None = _replace_once(
            value=block,
            pattern=rf'(?m)(^version\s*=\s*"){re.escape(old_version)}("[^\n]*$)',
            replacement=rf"\g<1>{new_version}\g<2>",
        )
        if replaced is None:
            return None
        blocks[index] = replaced
    return "".join(blocks) if found == owned_packages else None


def _replace_once(*, value: str, pattern: str, replacement: str) -> str | None:
    replaced, count = re.subn(pattern, replacement, value, count=1)
    return replaced if count == 1 else None


def _show(*, ref: str, path: str, repo_root: Path) -> str:
    try:
        return _git("show", f"{ref}:{path}", repo_root=repo_root)
    except subprocess.CalledProcessError:
        if path in OPTIONAL_BASE_PATHS:
            return ""
        raise


def _git(*arguments: str, repo_root: Path) -> str:
    result: subprocess.CompletedProcess[str] = subprocess.run(
        ["git", *arguments],
        cwd=repo_root,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def _parse_args() -> argparse.Namespace:
    parser: argparse.ArgumentParser = argparse.ArgumentParser(prog="release_metadata_check")
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", default="HEAD")
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--allow-stale-lockfiles", action="store_true")
    return parser.parse_args()


def main() -> int:
    """Validate one generated release delta and render actionable failures."""

    arguments: argparse.Namespace = _parse_args()
    delta: ReleaseDelta = load_release_delta(
        repo_root=arguments.repo,
        base_ref=arguments.base,
        head_ref=arguments.head,
    )
    errors: list[str] = get_release_delta_errors(
        delta=delta,
        allow_stale_lockfiles=arguments.allow_stale_lockfiles,
    )
    for error in errors:
        print(f"release metadata error: {error}")
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
