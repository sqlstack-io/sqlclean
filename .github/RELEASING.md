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

## Verify

`verify.yml` classifies a pull request by the paths it changes and runs only the suites it needs.
The `Verify` job is the one check to require: it passes when every selected suite passed.

| Change | Suites |
|--------|--------|
| Only `website/`, `benchmarks/`, root `*.md`, `.github/demos/`, images, `website.yml` | docs skill check and ruff |
| Also the bundled skill under `crates/sqlclean-cli/skill/` | plus a Linux build and the `sqlclean skills` tests |
| Anything else, including workflows, Rust, Python, tests and lockfiles | everything: `make verify`, minimum Rust, Windows, three-platform package smoke tests and the source build |

A manual or release dispatch has no pull request diff, so it runs everything.
