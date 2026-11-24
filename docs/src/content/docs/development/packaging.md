---
title: Package native bindings
description: Build, bundle and validate release wheels and Node.js platform packages.
---

The release workflow builds self-contained artifacts for four targets:

| Target | Wheel tag | Node.js package |
| --- | --- | --- |
| macOS arm64 | `macosx_26_0_arm64` | `@brimp/brimp-darwin-arm64` |
| Linux x86_64 | `manylinux_2_34_x86_64` | `@brimp/brimp-linux-x64-gnu` |
| Linux arm64 | `manylinux_2_34_aarch64` | `@brimp/brimp-linux-arm64-gnu` |
| Windows x64 | `win_amd64` | `@brimp/brimp-win32-x64` |

These are the native runtime build targets. Normal Python/Node installations
also require the WebKit companion, currently available only on macOS 26.0+ arm64
and Linux x64 with glibc 2.39+. The standalone CLI keeps the wider matrix above.

Intel macOS and Windows arm64 are not built. The pinned JavaScriptCore SDK
does not publish those artifacts. Linux wheels require glibc 2.34 or newer; the
Skia build depends on FreeType 2.10+.

## Pinned native SDKs

`scripts/prepare_native.py` downloads and verifies:

- JavaScriptCore from `webkit-v-27.2.post7` in `lexiforest/brimp-webkit`;
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
export MACOSX_DEPLOYMENT_TARGET=26.0
python scripts/prepare_native.py \
  --target aarch64-apple-darwin --output .native/release --curl-release
python -m pip install 'maturin>=1.9,<2'
maturin build --release --locked \
  --auditwheel repair --compatibility pypi \
  --manifest-path bindings/python/Cargo.toml --out dist/raw
```

Use `--compatibility manylinux_2_34` inside a manylinux_2_34 container for
Linux, then run `bindings/python/bundle_linux_runtime.py --wheel dist/raw/NAME.whl
--out dist/wheels` to include libatomic and its license notices. Manylinux treats
libatomic as a system library, so ordinary wheel repair alone does not bundle it.
On Windows, run `bindings/python/bundle_windows_runtime.py` after maturin
to add `msvcp140.dll`, `msvcp140_2.dll`, `vcruntime140.dll` and
`vcruntime140_1.dll` to `brimp.libs`.

Validate and clean-install without the build SDKs:

```sh
python bindings/python/validate_wheel.py \
  --target aarch64-apple-darwin --directory dist/wheels
python bindings/python/test_wheel.py \
  --target aarch64-apple-darwin --directory dist/wheels
```

`test_wheel.py` hides `.native`, removes the SDK environment, installs the base
wheel into a fresh virtual environment with `--no-index --no-deps`, and runs
`bindings/python/smoke.py`. This isolates the native payload before the required
companion has been built. Full dependency resolution is tested later by the
WebKit release matrix.

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

`bindings/test_node_distribution.py --directory dist/node` extracts and tests
the base runtime payload without resolving dependencies. This build check can
run before the first companion release exists. The WebKit release matrix uses
pip/npm to verify complete installations with the required companion.

## Standalone CLI archives

The CLI and Node package use `bindings/native_runtime.py` to bundle and relocate
the same runtime libraries. The CLI archive contains its executable, native
libraries, dependency licenses, and a manifest with file hashes. Linux and macOS
use tar.gz archives; Windows uses a ZIP with the DLLs beside `brimp.exe`.

Build and package the CLI using the SDK environment from [Build from source](/development/build-from-source/):

```sh
cargo build --release --locked -p brimp-cli
python bindings/package_cli.py \
  --target aarch64-apple-darwin \
  --jsc-sdk .native/release/jsc --curl-sdk .native/release/curl \
  --licenses .native/release/licenses --out dist/cli
python bindings/package_cli.py --validate --out dist/cli
```

Each archive has a `.sha256` sidecar. To test an archive, set `BRIMP_CLI_ARCHIVE`
to its path and run `python -m pytest bindings/test_cli_distribution.py`. The
tests unpack it into a temporary directory and hide the build SDKs while checking
runtime initialization, JavaScript, networking, and article extraction.

The release matrix builds the CLI alongside the bindings to reuse their compiled
dependencies. The publish job requires all four CLI archives and verifies their
manifests before uploading them. The installer at `https://brimp.ai/install.sh`
downloads the matching archive and checksum from the GitHub release.

Release Rust binaries strip symbols. The Node/CLI bundlers also strip copied
Unix SDK libraries before relocation and macOS signing.
Linux Python, Node and CLI packages explicitly include libatomic and the GCC
runtime license notices. The wheel postprocessor preserves the platform tag
and regenerates RECORD after adding the library and adjusting loader paths.

## Release workflow

`.github/workflows/release.yaml` handles two independent release types:

| Tag | Version source | Published artifacts |
| --- | --- | --- |
| `v0.1.2` | Cargo workspace and Brimp binding metadata | Brimp wheels, npm packages and CLI archives |
| `webkit-v0.1.0` | `version` in `tools/webkit/release.json` | brimp-webkit wheels and @brimp/webkit packages |

Brimp releases build and smoke-test the four platform distributions, then install
the new Python wheel in a clean environment. pip resolves its required
`brimp-webkit` dependency from PyPI, matching the user installation path. While
npm publishing is paused, these checks do not require a published Node companion
or GitHub release tarballs. Node builds and runtime tests remain enabled, and
companion candidate releases still test both Python and Node packages.
Brimp releases do not download a WebKit SDK, build its host, or repackage or
publish the companion. The separate JavaScriptCore SDK is still needed to build
Brimp's default runtime.

Companion releases build the host and package the pinned engine SDK for macOS
arm64 and Linux x64. They build current Brimp bindings as test dependencies and
test them against the candidate companion before publishing only the companion.
This allows the first companion release without an existing Brimp release.

Missing Windows WebKit binaries do not block either release type. Companion
artifact validation and full WebKit installation tests require macOS arm64 and
Linux x64 only. Windows JSC runtime builds and smoke tests remain required.
The required companion dependency remains unchanged; publishing Windows Brimp
artifacts does not make Python installation on Windows supported until a Windows
companion wheel is available.

Both release types validate their tag, write `SHA256SUMS`, publish wheels through
PyPI trusted publishing, and create a GitHub release. npm registry publication is
paused for both Brimp and its companion. Node packages are still built, tested,
and attached to GitHub Releases as `.tgz` archives; no `NPM_TOKEN` is required.
Companion releases do not replace the latest Brimp release used by the CLI installer.

PyPI trusted publishing must be configured for the `pypi` environment.

## WebKit companion packages

New WebKit and JavaScriptCore SDK releases follow curl-impersonate's archive
naming convention: `<product>-v<VERSION>.<architecture>-<platform>.tar.gz`.
For example, tag `webkit-v-27.2.post7` produces:

| SDK | Archive |
| --- | --- |
| WebKit Linux x86_64 | `webkit-v27.2.post7.x86_64-linux-gnu.tar.gz` |
| WebKit macOS arm64 | `webkit-v27.2.post7.arm64-macos.tar.gz` |
| JSC Linux x86_64 | `javascriptcore-v27.2.post7.x86_64-linux-gnu.tar.gz` |
| JSC Linux aarch64 | `javascriptcore-v27.2.post7.aarch64-linux-gnu.tar.gz` |
| JSC macOS arm64 | `javascriptcore-v27.2.post7.arm64-macos.tar.gz` |
| JSC Windows x86_64 | `javascriptcore-v27.2.post7.x86_64-win32.tar.gz` |

The Windows JSC symbols archive uses `.symbols.tar.gz` after the target.
Linux WebKit's build distribution stays in its SDK metadata (`build_os`),
rather than its archive name. Its ABI and system requirements still apply.
Both SDK pins now use the published `webkit-v-27.2.post7` release and these names.
The four JSC archive checksums are pinned in `scripts/prepare_native.py`.

All macOS builds target 26.0. The pinned post7 WebKit and JSC SDKs were built
for 26.0; their downloaded archives were verified against the release checksums.
The companion packager rejects SDKs with a different minimum instead of relabeling
them. The engine workflow and the Brimp release workflow both gate publication
on macOS 26 tests.

The Brimp repository publishes `brimp-webkit` to PyPI and distributes `@brimp/webkit`
and its two platform packages as GitHub release tarballs while npm publishing is
paused. They share the independent companion version in `tools/webkit/release.json`.
Brimp's required Python and Node dependencies
pin that companion version; `scripts/check_versions.py` checks those pins
without requiring the Brimp and companion versions to match. Multiple Brimp
releases can keep the same companion pin, avoiding duplicate engine releases and
downloads. Bump the companion version and both pins only when the engine, native
host, extraction code or packaged dependencies change.

The WebKit repository publishes engine SDK archives; the downstream
SDK tag is pinned in `tools/webkit/release.json` and verified against its release
checksums and file manifest. Both platforms must use the same source commit.

The WebKit release matrix currently targets macOS arm64 (the minimum OS comes
from the SDK's deployment target) and Linux x64, built on Ubuntu 24.04 and repaired
with auditwheel for `manylinux_2_39_x86_64`. Windows and Linux arm64 have no WebKit
companion package yet, so normal package installation currently requires one of
the supported WebKit platforms. The CLI and native runtime build matrix remain
broader so those binaries are ready when the companion adds more platforms.

The Ubuntu 24.04 packaging host explicitly installs `libavif16` and `libjxl0.7`,
which the pinned SDK links against. Current distro WebKit development packages
can pull newer codec ABIs instead. Auditwheel includes these codec libraries in
the companion payload.

```sh
pip install brimp
```

For Node, install the main and matching platform `.tgz` files for both Brimp and
its pinned companion from their GitHub Releases. Registry installation will be
available when npm publishing resumes.

Native binaries are stripped before packaging; Linux stripping precedes auditwheel
repair, and macOS code is re-signed afterward. Development headers, static
libraries, macOS linker stubs (`.tbd`) and debug bundles are omitted. The shared
engine payload uses XZ compression to keep wheel/npm downloads small.

The engine payload is included in each platform package. On first use it is
checksum-verified and extracted atomically into `$XDG_CACHE_HOME/brimp/webkit`
(or `~/.cache/brimp/webkit`). Extraction preserves framework symlinks and
executable permissions, works with read-only package directories, and performs
no network access. Python and Node use the same payload and cache key. Selecting
`engine="webkit"` or `engine: 'webkit'` discovers the companion automatically;
an explicit executable path still selects a development/custom host.

Linux also requires a display or Xvfb, a D-Bus session, bubblewrap,
xdg-dbus-proxy, fonts, and graphics drivers. These OS facilities are not pip/npm
packages. On Ubuntu 24.04, install them with:

```sh
sudo apt-get install xvfb xauth dbus-x11 bubblewrap xdg-dbus-proxy fonts-dejavu-core fontconfig libgl1-mesa-dri libegl1 libegl-mesa0 libgles2
```

For a headless application, create a private `XDG_RUNTIME_DIR` and run it under
`dbus-run-session -- xvfb-run -a`. Keep the WebKit process sandbox enabled.
The SDK's current media limitations still apply.

The release workflow builds the embedding host, bundles/signs macOS runtime
code or repairs Linux ELF dependencies, and tests installed Python and Node
packages while the input SDK path is hidden. Tests cover automatic discovery
and the shared WebKit navigation/input/cancellation contracts. Linux installation
tests run in an Ubuntu 24.04 container with only documented OS prerequisites; no
distribution WebKit/GTK libraries or development SDKs are installed. Publishing waits
for both platforms. The test image also removes any system libatomic installed
transitively by graphics packages, so tests must use the bundled copies. Publishing validates
versions, engine revisions and identical
Python/npm payload hashes. Manual workflow runs build and test without publishing;
select `package=brimp` to test with the published companion or `package=webkit`
to build and test a companion candidate.

Before the first release with WebKit:

1. Validate the engine release pinned in `tools/webkit/release.json`.
   The pinned `webkit-v-27.2.post7` includes the macOS trusted single-click fix.
   Build and run installed-package tests against these published artifacts
   before publishing Brimp.
   The earlier Ubuntu 26.04 fixed-prefix SDK is rejected by this pipeline.
2. Configure PyPI trusted publishing for **both** `brimp` and `brimp-webkit`,
   using this repository, `release.yaml`, and the `pypi` environment.
3. Run the Release workflow manually with `package=webkit`. Once its checks pass,
   push `webkit-v0.1.0` to publish its wheels to PyPI and its artifacts to GitHub Releases.
4. Run the workflow manually with `package=brimp` to test Brimp against that
   published companion. Once its checks pass, push `v0.1.2` to publish Brimp.
   Later Brimp versions use only their own `v*` tag while the companion is unchanged.

For version checks, use `python scripts/check_versions.py --tag v0.1.2` for Brimp
or `python scripts/check_versions.py --package webkit --tag webkit-v0.1.0` for
the companion. The engine repository's `webkit-v-27.2.post7` SDK tag is separate
from this repository's `webkit-v0.1.0` package tag.

Local development can use `tools/webkit/package.py --sdk PATH --out DIST
--development-sdk`. Such artifacts record development provenance and the release
validator refuses to publish them. Use `tools/webkit/test_distribution.py` with
base Brimp wheel/npm packages to exercise them outside the source tree.
