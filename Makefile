.PHONY: format format-check

format:
	npm run format
	ruff format .
	cargo fmt --all
	cargo fmt --manifest-path docs/tools/cli-reference/Cargo.toml

check:
	npm run format:check
	ruff format --check .
	cargo fmt --all -- --check
	cargo fmt --manifest-path docs/tools/cli-reference/Cargo.toml -- --check
