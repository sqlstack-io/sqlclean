SHELL := /bin/bash

.PHONY: check check-ci check-docs-skill check-rust develop-native skills test test-e2e test-integration test-rust test-unit verify

check:
	uv run ruff format .
	uv run ruff check --fix .
	uv run ty check src tests
	@if command -v cargo >/dev/null; then uv run fensu check; else uv run fensu check --target python; fi
	@command -v cargo >/dev/null && $(MAKE) --no-print-directory check-rust || true

check-ci:
	uv run ruff format --check .
	uv run ruff check .
	uv run ty check src tests
	uv run fensu check
	$(MAKE) --no-print-directory check-docs-skill
	$(MAKE) --no-print-directory check-rust

check-docs-skill:
	uv run python -m scripts.check_docs_skill

skills:
	uv run python -m scripts.generate_docs_skill

check-rust:
	cargo fmt --check
	cargo clippy --all-targets --quiet -- -D warnings

develop-native:
	uv sync --all-groups --reinstall-package sqlclean-cli

test: test-rust
	uv run pytest tests -q -n auto

test-rust:
	cargo test --all --quiet

test-unit:
	uv run pytest tests/unit -q -n auto

test-integration:
	uv run pytest tests/integration -q -n auto

test-e2e:
	uv run pytest tests/e2e -q -n auto

verify: check-ci test
