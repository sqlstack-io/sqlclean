# Releasing

The website and the packages ship separately.

## Website

Cloudflare Pages builds `website/` (`npm ci && npm run build`, output `dist`) from its own Git
integration. A push to `main` deploys https://sqlclean.io; every other branch deploys a preview
at `<branch>.sqlclean.pages.dev`. No workflow here deploys the site, and publishing packages never
touches it.

## Packages

Release Please keeps a release pull request open on `main`. Merging it tags `v0.MINOR.PATCH` and
dispatches `publish.yml`, which builds the `sqlclean-cli` wheels and both sdists and publishes
`sqlclean` and `sqlclean-cli` to PyPI.

Release Please counts every Conventional Commit on `main` since the last release, whatever its
type: `docs:` and `chore:` commits bump the patch version even though their changelog sections
are hidden or not. What keeps a change out of a release is `exclude-paths` in
`release-please-config.json`. A commit whose files all sit under `website/`, `benchmarks/` or
`.github/` is ignored, so site, benchmark and workflow edits never cut a release.

Documentation edits usually do release, on purpose: `make skills` copies the docs into
`crates/sqlclean-cli/skill/`, which the `sqlclean` executable embeds for `sqlclean skills`, so the
installed skill stays in step with the docs. A homepage or styling change touches only `website/`
and does not.

Merge pull requests one at a time, each with its branch up to date with `main`. The release
workflow only trusts a release base whose tree passed Verify on the pull request that produced
it. If two pull requests merge back to back, `main` holds a combination no pull request verified,
and the release workflow stops before it refreshes the release pull request's lockfiles. A later
run that leaves the release pull request unchanged does not repair it. To recover, close the
release pull request, then merge the next up-to-date pull request; Release Please opens a fresh
release pull request from the verified tree.

## One-time setup

A repository administrator does these once, in a browser:

1. **Release GitHub App.** Install the organization's delivery GitHub App on this repository,
   with contents, pull requests, issues and actions write access. In Settings > Secrets and
   variables > Actions, add the variable `RELEASE_APP_ID` (the App ID) and the secret
   `RELEASE_APP_PRIVATE_KEY` (the full contents of a private key generated under the app's
   Credentials). Release Please opens release pull requests and dispatches `publish.yml` with
   this app's token, because events from the default `GITHUB_TOKEN` do not start workflows.
   Without either setting, the Release Please run fails at its first step and names the missing
   one.
2. **PyPI trusted publishers.** For both `sqlclean` and `sqlclean-cli`, add a GitHub publisher
   with owner `sqlstack-io`, repository `sqlclean`, workflow `publish.yml` and environment
   `pypi`. GitHub creates the `pypi` environment the first time `publish.yml` uses it.
3. **Cloudflare Pages.** The `sqlclean` Pages project builds `website/` from this repository.
   Add `sqlclean.io` and `www.sqlclean.io` under its Custom domains.

Automation tokens, such as a coding agent's GitHub App token or `GITHUB_TOKEN`, usually cannot
read or write Actions variables, secrets or app installations, and a private key can only be
downloaded when it is generated. These steps need a person.

## Tooling versions

`uv.lock` is only reproducible with one uv release line: a newer uv rewrites the file's format,
and the release metadata check then rejects the release pull request. Workflows pin uv 0.12.9
and `[tool.uv] required-version` in `pyproject.toml` keeps local uv on 0.12, so update both
together. `make release-rehearsal` (the Release rehearsal job in Verify) applies Release Please's
version edits on a scratch worktree, refreshes the lockfiles the same way the release workflow
does, and runs the release metadata check, so a mismatch fails a pull request rather than a
release.

## Verify

`verify.yml` classifies a pull request by the paths it changes and runs only the suites it needs.
The `Verify` job is the one check to require: it passes when every selected suite passed.

| Change | Suites |
|--------|--------|
| Only `website/`, `benchmarks/`, root `*.md`, `.github/demos/`, images, `website.yml` | docs skill check and ruff |
| Also the bundled skill under `crates/sqlclean-cli/skill/` | plus a Linux build and the `sqlclean skills` tests |
| Anything else, including workflows, Rust, Python, tests and lockfiles | everything: `make verify`, minimum Rust, Windows, three-platform package smoke tests, the source build and the release rehearsal |

A manual or release dispatch has no pull request diff, so it runs everything.
