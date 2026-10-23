---
title: Development
description: Build one layer, run local checks and regenerate the documentation.
---

Use this workflow to make a change and verify the layer it affects.

## 1. Find the owning crate

![Three host interfaces share brimp-runtime; JavaScriptCore, Lexbor and curl-impersonate live behind independent lower crates.](/diagrams/runtime-layers.png)

| Change | Start here |
| --- | --- |
| DOM arena, parser or C boundary | `crates/lexbor` |
| Engine ownership, callbacks or protected values | `crates/jsc` |
| HTTP/WebSocket transport | `crates/net` |
| Browser APIs, sessions, scheduling or plugins | `crates/runtime` |
| Command-line behavior | `crates/cli` |
| Python/Node host interface | `bindings/python`, `bindings/node` |

## 2. Start with an independent layer

Lexbor needs no JSC or curl SDK:

```sh
cargo test -p brimp-lexbor
python3 scripts/test_lexbor_boundary.py
```

The second command exercises the native boundary under ASan/UBSan. JSC tests
require only the engine SDK; networking tests require only curl-impersonate:

```sh
cargo test -p brimp-jsc
cargo test -p brimp-net
```

## Native prerequisites

Set `BRIMP_JSC_LIB_DIR` and `BRIMP_CURL_LIB_DIR` as shown in
[installation](/install/#2-point-to-your-native-sdks). The repository also includes
`scripts/prepare_native.py` for pinned SDK downloads; inspect its target and
output options before selecting your platform:

```sh
python3 scripts/prepare_native.py --help
```

On macOS, local defaults are the adjacent `WebKit/WebKitBuild/Release` directory
and `/usr/local/lib` for curl. When using that WebKit build, run:

```sh
export DYLD_FRAMEWORK_PATH="$PWD/../WebKit/WebKitBuild/Release"
```

Lexbor 3.0.0 is vendored with provenance and documented patches. OpenSSL builds
from pinned Cargo source and needs a C compiler, make and Perl; no OpenSSL shared
library is required. Rendering, fonts, GPU/audio engines and browser Worker
execution are outside the runtime.

## 3. Build and test the host interfaces

```sh
cargo build -p brimp-cli -p brimp-python -p brimp-node
python3 bindings/prepare_local.py
cargo test --workspace
PYTHONPATH=bindings/python/python python3 -m pytest bindings/python/tests
node --test bindings/node/test/*.test.mjs
python3 -m pytest crates/cli/tests/test_cli.py
```

Install `pytest` in your Python environment first. Tests use local HTTP fixtures.
Re-run `prepare_local.py` after rebuilding bindings so Python and Node load the
new extensions.

## 4. Check the boundaries and style

```sh
python3 scripts/check_runtime_dependencies.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Use [the coverage audit](https://github.com/lexiforest/brimp/blob/main/compatibility/coverage/README.md)
for browser behavior probes. For performance work, use the reproducible
[native](https://github.com/lexiforest/brimp/blob/main/benchmark/native/README.md)
and [network](https://github.com/lexiforest/brimp/blob/main/benchmark/network/README.md)
benchmarks with the same build profile and host.

## 5. Regenerate docs and references

From the repository root, install the documentation dependencies into a virtual
environment, then build:

```sh
python3 -m venv /tmp/brimp-docs-env
. /tmp/brimp-docs-env/bin/activate
python -m pip install -r docs/requirements.txt
npm ci --prefix docs
npm run build --prefix docs
python -m pytest docs/tests/test_site.py
```

The build runs mkdocstrings on Python source, TypeDoc on the Node declarations,
and a small Rust helper on the CLI's existing Clap model. It renders editable
SVG diagram sources to PNG and builds the Starlight site. **No native SDK is
needed for documentation generation.**

To test the runnable tutorial code against the built bindings and CLI:

```sh
PYTHONPATH=bindings/python/python python3 -m pytest docs/tests/test_examples.py
```

Edit signatures and docstrings in their source files. Edit diagrams in
`docs/diagrams/`. Do not edit generated reference output by hand.
