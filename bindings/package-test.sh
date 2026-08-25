#!/bin/sh
set -eu

workspace=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
output="$workspace/dist"
test_root=$(mktemp -d "${TMPDIR:-/tmp}/brimp-package-test.XXXXXX")
trap 'rm -rf "$test_root"' EXIT

mkdir -p "$output/python" "$output/node"
uvx maturin build --manifest-path "$workspace/bindings/python/Cargo.toml" \
  --release --auditwheel repair --out "$output/python"
"$workspace/bindings/node/package.sh" "$output/node"

set -- "$output"/python/brimp-*.whl
if [ "$#" -ne 1 ] || [ ! -f "$1" ]; then
  echo "expected exactly one Brimp wheel in $output/python" >&2
  exit 1
fi
python_wheel=$1

set -- "$output"/node/brimp-brimp-*.tgz
if [ "$#" -ne 1 ] || [ ! -f "$1" ]; then
  echo "expected exactly one Brimp npm package in $output/node" >&2
  exit 1
fi
node_package=$1

mkdir -p "$test_root/artifacts" "$test_root/conformance"
cp "$python_wheel" "$node_package" "$test_root/artifacts/"
cp "$workspace/bindings/conformance/run.py" \
  "$workspace/bindings/conformance/scenario.py" \
  "$workspace/bindings/conformance/scenario.mjs" "$test_root/conformance/"
python_wheel="$test_root/artifacts/$(basename "$python_wheel")"
node_package="$test_root/artifacts/$(basename "$node_package")"

mkdir -p "$test_root/audit-python" "$test_root/audit-node"
unzip -q "$python_wheel" -d "$test_root/audit-python"
tar -xzf "$node_package" -C "$test_root/audit-node"
python_extension=$(find "$test_root/audit-python" -name '_brimp*.so' -type f)
node_extension="$test_root/audit-node/package/brimp_node.node"
for extension in "$python_extension" "$node_extension"; do
  dependencies=$(otool -L "$extension")
  case "$dependencies" in
    *"$workspace"*|*"/usr/local/lib/libcurl-impersonate"*)
      echo "packaged extension retains a build-machine dependency: $extension" >&2
      echo "$dependencies" >&2
      exit 1
      ;;
  esac
  file "$extension" | grep -q 'arm64'
done

python3 -m venv "$test_root/venv"
"$test_root/venv/bin/python" -m pip install --no-index "$python_wheel"
"$test_root/venv/bin/python" "$workspace/bindings/python/test_api.py"
mkdir -p "$test_root/node"
npm_config_cache="$test_root/npm-cache" npm install --prefix "$test_root/node" "$node_package"

BRIMP_PYTHON="$test_root/venv/bin/python" \
  python3 "$test_root/conformance/run.py" - \
  "$test_root/node/node_modules/@brimp/brimp"
