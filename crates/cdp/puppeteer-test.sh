#!/bin/sh
set -eu

workspace=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
fixture="$workspace/crates/cdp/puppeteer"
test_root=$(mktemp -d "${TMPDIR:-/tmp}/brimp-cdp-puppeteer.XXXXXX")
trap 'rm -rf "$test_root"' EXIT

cp "$fixture/package.json" "$fixture/package-lock.json" "$fixture/workflow.mjs" "$fixture/playwright-workflow.mjs" "$test_root/"
(cd "$test_root" && npm ci --ignore-scripts --no-audit --no-fund)
cargo build --manifest-path "$workspace/Cargo.toml" -p brimp-cli
python3 "$fixture/run.py" "$workspace/target/debug/brimp" "$test_root/workflow.mjs" "$test_root/playwright-workflow.mjs"
