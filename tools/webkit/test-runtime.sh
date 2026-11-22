#!/usr/bin/env bash
set -euo pipefail
export XDG_RUNTIME_DIR
XDG_RUNTIME_DIR=$(mktemp -d)
trap 'rm -rf "$XDG_RUNTIME_DIR"' EXIT
dbus-run-session -- xvfb-run -a python3 "$@"
