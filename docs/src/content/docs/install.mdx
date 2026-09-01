---
title: Install
description: Install the Python and Node bindings or build the Brimp CLI.
---

Choose the package that matches the interface you plan to use.

## Python

Brimp publishes self-contained CPython 3.10+ ABI3 wheels:

```sh
pip install brimp
```

Supported wheel targets are:

| Operating system | Architecture | Minimum/runtime target |
| --- | --- | --- |
| Linux | x86-64, ARM64 | manylinux 2.28 |
| macOS | ARM64 | macOS 11 or newer |
| Windows | x86-64 | 64-bit Windows |

The wheels carry JavaScriptCore, curl-impersonate, and the required non-system
runtime libraries. No separate browser download is required.

Verify the package:

```sh
python -c "import brimp; p=brimp.get('https://example.com'); print(p.status_code); p.close()"
```

## Node.js

The Node binding currently supports macOS on Apple silicon and requires Node.js
18 or newer:

```sh
npm install @brimp/brimp
```

The package is a native addon. Other Node platforms are not currently
published.

## CLI and CDP server

Each tagged GitHub Release includes standalone CLI archives for every supported
native target:

| Operating system | Architecture | Archive target |
| --- | --- | --- |
| Linux | x86-64 | `x86_64-unknown-linux-gnu` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` |
| macOS 11+ | ARM64 | `aarch64-apple-darwin` |
| Windows | x86-64 | `x86_64-pc-windows-msvc` |

Unix archives use `.tar.gz`; Windows uses `.zip`. Every archive has a matching
`.sha256` file and bundles JavaScriptCore, curl-impersonate, required non-system
runtime libraries, and licenses. For example, on macOS:

```sh
shasum -a 256 -c brimp-vVERSION-aarch64-apple-darwin.tar.gz.sha256
tar -xzf brimp-vVERSION-aarch64-apple-darwin.tar.gz
./brimp-vVERSION-aarch64-apple-darwin/brimp doctor
```

Other targets build the `brimp` executable, including the `brimp cdp`
subcommand, from source:

```sh
git clone https://github.com/lexiforest/brimp.git
cd brimp
cargo build --release -p brimp-cli
./target/release/brimp doctor
```

Source builds dynamically link JavaScriptCore and curl-impersonate. Point the
build at their library directories before invoking Cargo:

```sh
export BRIMP_JSC_LIB_DIR=/path/to/javascriptcore/lib
export BRIMP_CURL_LIB_DIR=/path/to/curl-impersonate/lib
cargo build --release -p brimp-cli
```

Expected library layouts vary by platform:

- macOS: `JavaScriptCore.framework` and a curl-impersonate dynamic library;
- Linux: `libJavaScriptCore.so` and a curl-impersonate shared library;
- Windows: `JavaScriptCore.lib` and the corresponding curl import/runtime libraries.

After installation, continue with the [quick start](/quick-start/).
