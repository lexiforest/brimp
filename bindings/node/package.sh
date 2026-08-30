#!/bin/sh
set -eu

workspace=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
output=${1:-"$workspace/dist/node"}
staging=$(mktemp -d "${TMPDIR:-/tmp}/brimp-node.XXXXXX")
trap 'rm -rf "$staging"' EXIT

cargo build --manifest-path "$workspace/Cargo.toml" --release -p brimp-node
cp "$workspace/bindings/node/index.js" "$workspace/bindings/node/index.d.ts" \
  "$workspace/bindings/node/package.json" "$workspace/bindings/node/README.md" \
  "$workspace/bindings/node/SUPPORT.md" "$staging/"
cp "$workspace/target/release/libbrimp_node.dylib" "$staging/brimp_node.node"
cp /usr/local/lib/libcurl-impersonate.4.8.0.dylib "$staging/libcurl-impersonate.4.dylib"
mkdir -p "$staging/licenses"
cp "$workspace/LICENSE" "$staging/licenses/brimp-LICENSE"
cp "$workspace/bindings/python/python/brimp/licenses/curl-impersonate-LICENSE" \
  "$staging/licenses/"
mkdir -p "$staging/licenses/defuddle"
cp "$workspace/crates/web-runtime/vendor/defuddle/0.19.3/NOTICE.md" \
  "$staging/licenses/defuddle/"
cp -R "$workspace/crates/web-runtime/vendor/defuddle/0.19.3/licenses" \
  "$staging/licenses/defuddle/"
install_name_tool -change @rpath/libcurl-impersonate.4.dylib \
  @loader_path/libcurl-impersonate.4.dylib "$staging/brimp_node.node"
install_name_tool -id @rpath/brimp_node.node "$staging/brimp_node.node"
mkdir -p "$output"
(cd "$staging" && npm_config_cache="$staging/.npm-cache" npm pack --pack-destination "$output")
