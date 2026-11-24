#!/usr/bin/env bash
set -euo pipefail
# Exercise the same unprivileged namespace setup WebKit needs before installing
# packages, so host policy failures are reported at their source.
if ! bwrap --unshare-user --unshare-pid --ro-bind / / --proc /proc true; then
    echo 'Bubblewrap user namespaces are blocked; check the host AppArmor policy.' >&2
    cat /proc/self/attr/current /proc/self/uid_map >&2
    exit 1
fi
export XDG_RUNTIME_DIR
XDG_RUNTIME_DIR=$(mktemp -d)
trap 'rm -rf "$XDG_RUNTIME_DIR"' EXIT
dbus-run-session -- xvfb-run -a python3 "$@"
