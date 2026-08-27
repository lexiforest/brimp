---
title: Install
description: Install the Python and Node bindings or build the Brimp CLI.
---

Choose the package that matches the interface you plan to use.

## Python

Brimp publishes self-contained CPython 3.10+ ABI3 wheels:

```sh
python -m pip install brimp
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
python -c "import brimp; print(brimp.get('https://example.com').status_code)"
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

The `brimp` executable, including the `brimp cdp` subcommand, is currently
produced by a source build:

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
