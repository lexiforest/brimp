#!/usr/bin/env bash
set -euo pipefail

# Native development dependencies for the Ubuntu 26.04 x86_64 node.
[[ $EUID -eq 0 ]] || { echo 'Run setup.sh with sudo or as root.' >&2; exit 1; }
# shellcheck source=/dev/null
. /etc/os-release
[[ $ID == ubuntu && $VERSION_ID == 26.04 && $(uname -m) == x86_64 ]] || {
    echo 'Native development targets Ubuntu 26.04 x86_64.' >&2; exit 1;
}
root=$(cd -- "$(dirname -- "$0")/../.." && pwd)
dev_user=${1:-brimp}
if ! id "$dev_user" >/dev/null 2>&1; then useradd --create-home --shell /bin/bash "$dev_user"; fi
[[ $(id -u "$dev_user") -ne 0 ]] || { echo 'Choose a non-root development user.' >&2; exit 1; }

cat > /etc/apt/sources.list.d/brimp-build-sources.list <<EOF
deb-src http://archive.ubuntu.com/ubuntu $VERSION_CODENAME main universe multiverse restricted
deb-src http://archive.ubuntu.com/ubuntu $VERSION_CODENAME-updates main universe multiverse restricted
deb-src http://security.ubuntu.com/ubuntu $VERSION_CODENAME-security main universe multiverse restricted
EOF
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get build-dep -y --no-install-recommends webkit2gtk
apt-get install -y --no-install-recommends \
    clang-18 lld-18 ccache cmake ninja-build unifdef nasm \
    build-essential libfontconfig1-dev libfreetype6-dev nlohmann-json3-dev \
    curl pkg-config git rsync ca-certificates python3-venv nodejs npm \
    xvfb xauth dbus-x11 bubblewrap xdg-dbus-proxy fonts-dejavu-core \
    libgl1-mesa-dri libegl1 libegl-mesa0 libgles2 libatomic1 apparmor

# Keep the system-wide restriction; enable only the distro's Bubblewrap policy
# if the normal user's sandbox preflight needs it. Ubuntu 26.04 ships this
# persistent policy in /etc/apparmor.d as part of the apparmor package.
if ! runuser -u "$dev_user" -- bwrap --unshare-user --unshare-pid --ro-bind / / --proc /proc true; then
    apparmor_parser -r /etc/apparmor.d/bwrap-userns-restrict
    runuser -u "$dev_user" -- bwrap --unshare-user --unshare-pid --ro-bind / / --proc /proc true
fi
toolchain=$(python3 -c 'import sys,tomllib; print(tomllib.load(open(sys.argv[1], "rb"))["toolchain"]["channel"])' "$root/rust-toolchain.toml")
runuser -u "$dev_user" -- bash -s -- "$toolchain" <<'USER_SETUP'
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
if [[ ! -x "$HOME/.cargo/bin/rustup" ]]; then
    installer=$(mktemp)
    trap 'rm -f "$installer"' EXIT
    curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs -o "$installer"
    sh "$installer" -y --profile minimal --default-toolchain none --no-modify-path
fi
rustup toolchain install "$1" --profile minimal --component clippy --component rustfmt
venv="$HOME/.local/share/brimp-dev/venv"
python3 -m venv "$venv"
"$venv/bin/python" -m pip install 'pytest>=8,<10' 'websockets>=12,<17' 'h2>=4,<5'
node -e 'const [a,b]=process.versions.node.split(".").map(Number); if (a<22 || (a===22 && b<19)) throw Error("Node >=22.19 required")'
USER_SETUP
echo "Ready. Run tools/linux/run.sh as $dev_user; no Docker is required."
