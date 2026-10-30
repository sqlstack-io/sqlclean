#!/usr/bin/env bash
# Rehearse a release on a scratch worktree: apply Release Please's version edits, refresh the
# lockfiles the way release-please.yml does, then run the strict release metadata check.
# This catches lockfile drift, such as a different uv rewriting uv.lock, before a real release.
set -euo pipefail

repo="$(git rev-parse --show-toplevel)"
current="$(jq -r '."."' "${repo}/.release-please-manifest.json")"
IFS=. read -r major minor _ <<< "${current}"
next="${major}.$((minor + 1)).0"
scratch="$(mktemp -d)"
trap 'git -C "${repo}" worktree remove --force "${scratch}" >/dev/null 2>&1 || true' EXIT

git -C "${repo}" worktree add --detach --quiet "${scratch}" HEAD
cd "${scratch}"
# Release Please rewrites the version on every line marked x-release-please-version.
for file in pyproject.toml Cargo.toml packages/sqlclean-cli/pyproject.toml; do
  sed -i "/x-release-please-version/s/${current//./\\.}/${next}/" "${file}"
done
# The Python release strategy also rewrites [project] version, which carries no marker.
sed -i "0,/^version = \"${current//./\\.}\"/s//version = \"${next}\"/" pyproject.toml
jq --arg next "${next}" '."." = $next' .release-please-manifest.json > manifest.tmp
mv manifest.tmp .release-please-manifest.json
# Release Please adds the new section under the title and keeps the earlier history.
{
  printf '# Changelog\n\n## [%s](https://example.com/compare/v%s...v%s) (2000-01-01)\n\n' \
    "${next}" "${current}" "${next}"
  tail -n +3 CHANGELOG.md
} > changelog.tmp
mv changelog.tmp CHANGELOG.md
# The same refresh as release-please.yml; cargo metadata rewrites Cargo.lock without compiling.
uv lock --quiet
cargo metadata --format-version 1 > /dev/null
git -c commit.gpgsign=false -c user.name=release-rehearsal -c user.email=release-rehearsal@example.com \
  commit --quiet --all --message "chore(main): release ${next}"
python3 "${repo}/.github/scripts/release_metadata_check.py" --repo "${scratch}" --base HEAD~1 --head HEAD
echo "Release rehearsal ${current} -> ${next} passed with $(uv --version)"
