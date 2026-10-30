---
title: Package native bindings
description: Build, bundle and validate release wheels and Node.js platform packages.
---

The release workflow builds self-contained artifacts for four targets:

| Target | Wheel tag | Node.js package |
| --- | --- | --- |
| macOS arm64 | `macosx_11_0_arm64` | `@brimp/brimp-darwin-arm64` |
| Linux x86_64 | `manylinux_2_34_x86_64` | `@brimp/brimp-linux-x64-gnu` |
| Linux arm64 | `manylinux_2_34_aarch64` | `@brimp/brimp-linux-arm64-gnu` |
| Windows x64 | `win_amd64` | `@brimp/brimp-win32-x64` |

Intel macOS and Windows arm64 are not built. The pinned JavaScriptCore SDK
does not publish those artifacts. Linux wheels require glibc 2.34 or newer; the
Skia build depends on FreeType 2.10+.

## Pinned native SDKs

`scripts/prepare_native.py` downloads and verifies:

- JavaScriptCore `jsc-v2026082706` from `lexiforest/WebKit`;
- curl-impersonate `v2.3.2` from `lexiforest/curl-impersonate`, including its
  BoringSSL WebCrypto interface.

The release bundles the SDK shared libraries, ICU, the Visual C++ runtime DLLs
used by the Windows JSC SDK, and the SDK licenses. `BRIMP_LICENSE_DIR` points
the binding build scripts at the collected license tree; both Python and Node
packages include it under `bundled/licenses/native` (the Node platform package
also carries a copy under `licenses/`).

## Python wheels

Build a wheel with maturin. `--auditwheel repair` performs auditwheel repair on
Linux, delocate on macOS and delvewheel on Windows:

```sh
python scripts/prepare_native.py \
  --target aarch64-apple-darwin --output .native/release --curl-release
python -m pip install 'maturin>=1.9,<2'
maturin build --release --locked \
  --auditwheel repair --compatibility pypi \
  --manifest-path bindings/python/Cargo.toml --out dist/raw
```

Use `--compatibility manylinux_2_34` inside a manylinux_2_34 container for
Linux. On Windows, run `bindings/python/bundle_windows_runtime.py` after maturin
to add `msvcp140.dll`, `msvcp140_2.dll`, `vcruntime140.dll` and
`vcruntime140_1.dll` to `brimp.libs`.

Validate and clean-install without the build SDKs:

```sh
python bindings/python/validate_wheel.py \
  --target aarch64-apple-darwin --directory dist/wheels
python bindings/python/test_wheel.py \
  --target aarch64-apple-darwin --directory dist/wheels
```

`test_wheel.py` hides `.native`, removes the SDK environment, installs the wheel
into a fresh virtual environment with `--no-index`, and runs
`bindings/python/smoke.py`.

## Node.js packages

The `@brimp/brimp` wrapper selects a native platform package at runtime and
falls back to a local `brimp.node` for development builds. Each platform archive
contains `brimp.node`, the native libraries and licenses. Build and pack it from
the prepared SDK:

```sh
cargo build --release --locked -p brimp-node
python bindings/prepare_local.py --release --only node
python bindings/package_node.py \
  --jsc-sdk .native/release/jsc \
  --curl-sdk .native/release/curl \
  --package --main \
  --out dist/node
```

macOS rewrites install names to `@loader_path/native`, Linux rewrites RPATH to
`$ORIGIN/native` with `patchelf`, and Windows places the DLLs next to
`brimp.node`. `--main` packs the JavaScript wrapper without a native binary;
that package declares the four platform packages as optional dependencies.

Validate an artifact set:

```sh
python bindings/package_node.py --validate --all --out dist/node
```

## Release workflow

`.github/workflows/release.yml` runs on `v*` tags. It builds the four wheels and
the four platform packages, installs and smoke-tests them, verifies the tag
matches the workspace version, writes `SHA256SUMS`, publishes wheels to PyPI
with trusted publishing, publishes npm packages with provenance, and creates a
GitHub release.

PyPI trusted publishing must be configured for the `pypi` environment and the
`NPM_TOKEN` repository secret must be able to publish the `@brimp` scope.
