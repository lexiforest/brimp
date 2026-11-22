#!/usr/bin/env bash
set -euo pipefail

# Run once on a new Ubuntu x86_64 node, or again to refresh the cached images.
[[ $EUID -eq 0 ]] || { echo 'Run setup-packaging.sh with sudo or as root.' >&2; exit 1; }
[[ $(uname -m) == x86_64 ]] || { echo 'Linux WebKit currently targets x86_64.' >&2; exit 1; }
# shellcheck source=/dev/null
. /etc/os-release
[[ $ID == ubuntu && ($VERSION_ID == 24.04 || $VERSION_ID == 26.04) ]] || {
    echo 'Use an Ubuntu 24.04 or 26.04 host.' >&2; exit 1;
}
root=$(cd -- "$(dirname -- "$0")/../.." && pwd)
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y --no-install-recommends docker.io git rsync ca-certificates python3
systemctl enable --now docker
toolchain=$(python3 -c 'import sys,tomllib; print(tomllib.load(open(sys.argv[1], "rb"))["toolchain"]["channel"])' "$root/rust-toolchain.toml")
docker build --build-arg "RUST_TOOLCHAIN=$toolchain" \
    -t brimp-linux-build -f "$root/tools/linux/Dockerfile" "$root/tools/linux"
docker build -t brimp-linux-runtime \
    -f "$root/tools/webkit/runtime.Dockerfile" "$root/tools/webkit"

echo 'Ready. Run tools/linux/run-packaging.sh doctor WEBKIT_SOURCE WORK_DIRECTORY.'
