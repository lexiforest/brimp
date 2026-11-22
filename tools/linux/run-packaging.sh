#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo 'Usage: run-packaging.sh {doctor|engine|bindings|package|test|all} WEBKIT_SOURCE WORK_DIRECTORY' >&2
    exit 2
}
[[ $# == 3 ]] || usage
stage=$1
case "$stage" in doctor|engine|bindings|package|test|all) ;; *) usage ;; esac
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || {
    echo 'Run this script on the Linux x86_64 build node.' >&2; exit 1;
}
root=$(cd -- "$(dirname -- "$0")/../.." && pwd)
webkit=$(realpath "$2")
work=$(realpath -m "$3")
[[ -f "$webkit/Tools/Scripts/linux-webkit/build.sh" ]]
# The workspace is generated content; never place it in either input checkout.
for source in "$root" "$webkit"; do
    [[ "$work/" != "$source/"* && "$source/" != "$work/"* ]] || {
        echo 'Use a work directory separate from both source checkouts.' >&2; exit 1;
    }
done
build_uid=$(id -u)
build_gid=$(id -g)
if [[ $EUID -eq 0 ]]; then build_uid=1000; build_gid=1000; fi
mkdir -p "$work/logs" "$work/home"
if [[ $EUID -eq 0 ]]; then chown "$build_uid:$build_gid" "$work" "$work/logs" "$work/home"; fi
jobs=${BRIMP_BUILD_JOBS:-$(awk '/MemTotal:/ { n=int($2/3145728); print n<1?1:n }' /proc/meminfo)}
[[ $jobs =~ ^[1-9][0-9]*$ ]] || { echo 'BRIMP_BUILD_JOBS must be positive.' >&2; exit 1; }
if (( jobs > $(nproc) )); then jobs=$(nproc); fi

build() {
    # Privileged Docker permits nested user namespaces. Browser processes still
    # run as an unprivileged user with WebKit's own sandbox enabled.
    docker run --rm --privileged --user "$build_uid:$build_gid" \
        --mount "type=bind,src=$root,dst=/input/brimp,readonly" \
        --mount "type=bind,src=$webkit,dst=$webkit,readonly" \
        --mount "type=bind,src=$work,dst=$work" \
        -e "HOME=$work/home" -e "BRIMP_BUILD_JOBS=$jobs" \
        brimp-linux-build bash /input/brimp/tools/linux/package-build.sh "$1" "$webkit" "$work"
}
test_packages() {
    # Only artifacts and the packaged SDK enter this container, never the
    # original engine install/build or the Rust/native SDK caches.
    docker run --rm --privileged --user "$build_uid:$build_gid" \
        --mount "type=bind,src=$root/tools/webkit,dst=/brimp/tools/webkit,readonly" \
        --mount "type=bind,src=$root/bindings/python/tests,dst=/brimp/bindings/python/tests,readonly" \
        --mount "type=bind,src=$root/bindings/node/test,dst=/brimp/bindings/node/test,readonly" \
        --mount "type=bind,src=$work/dist,dst=/artifacts,readonly" \
        --mount "type=bind,src=$work/package,dst=/sdk" \
        -e HOME=/tmp -e NPM_CONFIG_CACHE=/tmp/npm-cache \
        brimp-linux-runtime /brimp/tools/webkit/test_distribution.py \
        --dist /artifacts/webkit --base-wheels /artifacts/base-wheels \
        --base-node /artifacts/base-node --hide-sdk /sdk/webkit
}
{
    if [[ $stage == all ]]; then
        build engine
        build bindings
        build package
        test_packages
    elif [[ $stage == test ]]; then
        test_packages
    else
        build "$stage"
    fi
} 2>&1 | tee "$work/logs/$stage-$(date -u +%Y%m%dT%H%M%SZ).log"
