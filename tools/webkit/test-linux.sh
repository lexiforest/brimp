#!/usr/bin/env bash
set -euo pipefail

root=$(cd -- "$(dirname -- "$0")/../.." && pwd)
export BRIMP_WEBKIT_EXECUTABLE
BRIMP_WEBKIT_EXECUTABLE=$(realpath "${1:?usage: test-linux.sh HOST_EXECUTABLE}")
[[ -x "$BRIMP_WEBKIT_EXECUTABLE" ]]
cd "$root"
export PYTHONPATH="$root/bindings/python/python"
export GTK_A11Y=none LIBGL_ALWAYS_SOFTWARE=1
export XDG_RUNTIME_DIR
XDG_RUNTIME_DIR=$(mktemp -d)
trap 'rm -rf "$XDG_RUNTIME_DIR"' EXIT

# Set the native SDK environment and build the bindings before invoking this
# script. Every WebKit contract is enabled; a missing or broken host fails.
dbus-run-session -- xvfb-run -a "${PYTHON:-python3}" -m pytest \
    bindings/python/tests/test_browser_backend.py -v
dbus-run-session -- xvfb-run -a node --test bindings/node/test/browser-backend.test.mjs
