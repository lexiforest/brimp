#!/usr/bin/env bash
set -euo pipefail

# Container implementation. Use run-packaging.sh on the host.
stage=${1:?}
webkit=${2:?}
work=${3:?}
[[ $EUID -ne 0 ]] || { echo 'Build and test as a normal user.' >&2; exit 1; }
export CARGO_HOME="$work/cache/cargo" CCACHE_DIR="$work/cache/ccache"
export CXX=clang++-18 CARGO_BUILD_JOBS="$BRIMP_BUILD_JOBS"
mkdir -p "$CARGO_HOME" "$CCACHE_DIR" "$work/src" "$work/dist"
bwrap --unshare-user --unshare-pid --ro-bind / / --proc /proc true
if [[ $stage == doctor ]]; then
    clang-18 --version
    rustc --version
    python3 --version
    node --version
    python3 -c 'import pytest, wheel, auditwheel, websockets, h2'
    df -h "$work"
    echo "Ready: $BRIMP_BUILD_JOBS parallel build jobs; Bubblewrap sandbox works."
    exit
fi
if [[ $stage == engine ]]; then
    bash "$webkit/Tools/Scripts/linux-webkit/build.sh" "$webkit" "$work"
    bash "$webkit/Tools/Scripts/linux-webkit/test.sh" \
        "$work/sdk/webkit-curl" "$work/sdk/curl-impersonate-2.3.2"
    exit
fi

# Preserve compiler/download caches but never modify the input Brimp checkout.
brimp="$work/src/brimp"
mkdir -p "$brimp"
rsync -a --delete --no-owner --no-group \
    --exclude=.git --exclude=target --exclude=.native --exclude=.venv \
    --exclude=node_modules --exclude=__pycache__ --exclude=.cache \
    --exclude=dist --exclude='*.so' --exclude='*.node' --exclude=bundled \
    /input/brimp/ "$brimp/"
cd "$brimp"
if [[ $stage == bindings ]]; then
    python3 scripts/prepare_native.py --target x86_64-unknown-linux-gnu \
        --output .native/release --curl-release
    export BRIMP_JSC_SDK="$brimp/.native/release/jsc"
    export BRIMP_CURL_SDK="$brimp/.native/release/curl"
    export BRIMP_JSC_LIB_DIR="$BRIMP_JSC_SDK/lib" BRIMP_CURL_LIB_DIR="$BRIMP_CURL_SDK/lib"
    export BRIMP_LICENSE_DIR="$brimp/.native/release/licenses"
    export LD_LIBRARY_PATH="$BRIMP_JSC_LIB_DIR:$BRIMP_CURL_LIB_DIR"
    rm -rf "$work/dist/raw-wheels" "$work/dist/base-wheels" "$work/dist/base-node"
    maturin build --release --locked --auditwheel repair --compatibility manylinux_2_39 \
        --manifest-path bindings/python/Cargo.toml --out "$work/dist/raw-wheels"
    python3 bindings/python/bundle_linux_runtime.py --wheel "$work"/dist/raw-wheels/*.whl \
        --out "$work/dist/base-wheels"
    cargo build --release --locked -p brimp-node
    python3 bindings/prepare_local.py --release --only node
    python3 bindings/package_node.py --jsc-sdk "$BRIMP_JSC_SDK" --curl-sdk "$BRIMP_CURL_SDK" \
        --package --main --out "$work/dist/base-node"
elif [[ $stage == package ]]; then
    # Development provenance: even a tag checkout is not a verified release SDK.
    revision=$(git -c safe.directory="$webkit" -C "$webkit" rev-parse HEAD)
    mkdir -p "$work/package"
    rm -rf "$work/package/webkit" "$work/dist/webkit"
    python3 "$webkit/Tools/Scripts/package-webkit-sdk.py" \
        --platform linux --build-dir "$work/sdk/webkit-curl" \
        --output-dir "$work/package/webkit" --curl-build-dir "$work/build/curl" \
        --curl-source-dir "$work/src/curl-impersonate" \
        --tag "webkit-v-local-${revision:0:12}" --commit "$revision"
    python3 tools/webkit/package.py --sdk "$work/package/webkit" \
        --development-sdk --out "$work/dist/webkit"
else
    echo "Unknown build stage: $stage" >&2
    exit 2
fi
