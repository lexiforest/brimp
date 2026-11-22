#!/usr/bin/env bash
set -euo pipefail

[[ $# == 3 ]] || { echo 'Usage: run.sh {doctor|engine|bindings|host|test|all} WEBKIT_SOURCE WORK_DIRECTORY' >&2; exit 2; }
stage=$1
case "$stage" in doctor|engine|bindings|host|test|all) ;; *) echo "Unknown stage: $stage" >&2; exit 2 ;; esac
[[ $EUID -ne 0 ]] || { echo 'Run as the development user (brimp by default).' >&2; exit 1; }
# shellcheck source=/dev/null
. /etc/os-release
[[ $ID == ubuntu && $VERSION_ID == 26.04 && $(uname -m) == x86_64 ]]
root=$(cd -- "$(dirname -- "$0")/../.." && pwd)
webkit=$(realpath "$2")
work=$(realpath -m "$3")
[[ -f "$webkit/Tools/Scripts/linux-webkit/build.sh" ]]
export PATH="$HOME/.local/share/brimp-dev/venv/bin:$HOME/.cargo/bin:$PATH"
export PYTHON="$HOME/.local/share/brimp-dev/venv/bin/python"
[[ -x "$PYTHON" ]] || { echo 'Run tools/linux/setup.sh first.' >&2; exit 1; }
export BRIMP_BUILD_JOBS=${BRIMP_BUILD_JOBS:-$(awk '/MemTotal:/ { n=int($2/3145728); print n<1?1:n }' /proc/meminfo)}
[[ $BRIMP_BUILD_JOBS =~ ^[1-9][0-9]*$ ]]
if (( BRIMP_BUILD_JOBS > $(nproc) )); then BRIMP_BUILD_JOBS=$(nproc); fi
export CARGO_BUILD_JOBS="$BRIMP_BUILD_JOBS" CCACHE_DIR="$work/cache/ccache" CXX=clang++-18
mkdir -p "$work/logs" "$CCACHE_DIR"
cd "$root"

native_environment() {
    export BRIMP_JSC_LIB_DIR="$root/.native/development/jsc/lib"
    export BRIMP_CURL_LIB_DIR="$root/.native/development/curl/lib"
    export BRIMP_LICENSE_DIR="$root/.native/development/licenses"
    export LD_LIBRARY_PATH="$BRIMP_JSC_LIB_DIR:$BRIMP_CURL_LIB_DIR"
}
run_stage() {
    case "$1" in
        doctor)
            clang-18 --version
            rustc --version
            "$PYTHON" --version
            node --version
            "$PYTHON" -c 'import pytest, websockets, h2'
            df -h "$work"
            ;;
        engine)
            bash "$webkit/Tools/Scripts/linux-webkit/build.sh" "$webkit" "$work"
            bash "$webkit/Tools/Scripts/linux-webkit/test.sh" "$work/sdk/webkit-curl" "$work/sdk/curl-impersonate-2.3.2"
            ;;
        bindings)
            "$PYTHON" scripts/prepare_native.py --target x86_64-unknown-linux-gnu \
                --output .native/development --curl-release
            native_environment
            PYO3_PYTHON="$PYTHON" cargo build --release --locked -p brimp-python -p brimp-node
            "$PYTHON" bindings/prepare_local.py --release
            ;;
        host)
            "$PYTHON" tools/webkit/build.py --sdk "$work/sdk/webkit-curl"
            ;;
        test)
            native_environment
            bash tools/webkit/test-linux.sh "$root/target/webkit/brimp-webkit"
            ;;
    esac
}
{
    bwrap --unshare-user --unshare-pid --ro-bind / / --proc /proc true
    if [[ $stage == all ]]; then
        for step in engine bindings host test; do run_stage "$step"; done
    else
        run_stage "$stage"
    fi
} 2>&1 | tee "$work/logs/native-$stage-$(date -u +%Y%m%dT%H%M%SZ).log"
