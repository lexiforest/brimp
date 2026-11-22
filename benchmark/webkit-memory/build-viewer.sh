#!/usr/bin/env bash
set -euo pipefail
sdk=$(realpath "${1:?usage: build-viewer.sh SDK gtk|wpe OUTPUT}")
port=${2:?}
output=${3:?}
root=$(cd -- "$(dirname -- "$0")" && pwd)
export PKG_CONFIG_PATH="$sdk/lib/pkgconfig"
flags=()
if [[ "$port" == wpe ]]; then
    package=wpe-webkit-2.0
    flags+=(-DBENCH_WPE)
    platform=wpe-platform-headless-2.0
else
    package=webkitgtk-6.0
    platform=""
fi
[[ $(pkg-config --variable=prefix "$package") == "$sdk" ]]
# pkg-config returns compiler arguments, intentionally split into an array.
read -r -a dependencies <<< "$(pkg-config --cflags --libs "$package" ${platform:+"$platform"})"
clang++ -std=c++20 -O2 -Wall -Wextra "${flags[@]}" "$root/viewer.cpp" \
    "${dependencies[@]}" -ldl -o "$output"
