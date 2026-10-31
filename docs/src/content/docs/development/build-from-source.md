---
title: Build from source
description: Prepare native SDKs and build the CLI, Python binding, or Node.js binding.
---

Use these instructions to build the CLI or change Brimp itself. For normal
Python and Node.js use, [install a package](/install/).

## Prerequisites

You need Git, Rust/Cargo, Python 3.11 or newer, and a C/C++ compiler. The checkout
pins its Rust toolchain in `rust-toolchain.toml`. Install Node.js 22.19 or newer
if you want the Node binding.

On macOS, install the Xcode command-line tools. On Windows, use Visual Studio
2022 Build Tools with the C++ tools and Windows SDK. On Ubuntu, install the
native build dependencies:

```sh
sudo apt-get update
sudo apt-get install -y build-essential clang ninja-build pkg-config libfontconfig1-dev libfreetype6-dev
```

## 1. Get the source

```sh
git clone https://github.com/lexiforest/brimp.git
cd brimp
```

Run the remaining source-build commands from this directory.

## 2. Download the native SDKs

Choose the target for your machine:

| Platform | Target | Minimum system |
| --- | --- | --- |
| macOS Apple silicon | `aarch64-apple-darwin` | macOS 11 |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | glibc 2.34 |
| Linux arm64 | `aarch64-unknown-linux-gnu` | glibc 2.34 |
| Windows x64 | `x86_64-pc-windows-msvc` | Windows with the MSVC toolchain |

The helper downloads and verifies pinned JavaScriptCore and curl-impersonate
SDKs. For macOS Apple silicon:

```sh
python3 scripts/prepare_native.py --target aarch64-apple-darwin --output .native/release --curl-release
```

Replace the target for Linux. In Windows PowerShell, use `python` and the Windows
target:

```powershell
python scripts/prepare_native.py --target x86_64-pc-windows-msvc --output .native/release --curl-release
```

## 3. Set the SDK environment

The helper prints `NAME=value` lines. Outside GitHub Actions it does **not** set
variables in your shell. Export the paths before building or running Brimp, and
repeat this step in each new terminal where you use the source build.

On macOS and Linux:

```sh
export BRIMP_JSC_LIB_DIR="$PWD/.native/release/jsc/lib"
export BRIMP_CURL_LIB_DIR="$PWD/.native/release/curl/lib"
export BRIMP_LICENSE_DIR="$PWD/.native/release/licenses"
```

On macOS, also select the downloaded framework at runtime:

```sh
export DYLD_FRAMEWORK_PATH="$BRIMP_JSC_LIB_DIR"
export DYLD_LIBRARY_PATH="$BRIMP_JSC_LIB_DIR:$BRIMP_CURL_LIB_DIR"
```

On Linux, set the shared-library search path:

```sh
export LD_LIBRARY_PATH="$BRIMP_JSC_LIB_DIR:$BRIMP_CURL_LIB_DIR"
```

On Windows PowerShell:

```powershell
$env:BRIMP_JSC_LIB_DIR = "$PWD/.native/release/jsc/lib"
$env:BRIMP_CURL_LIB_DIR = "$PWD/.native/release/curl/lib"
$env:BRIMP_LICENSE_DIR = "$PWD/.native/release/licenses"
$env:BRIMP_NATIVE_RUNTIME_DIRS = "$PWD/.native/release/jsc/bin;$PWD/.native/release/curl/lib"
$env:PATH = "$env:BRIMP_NATIVE_RUNTIME_DIRS;$env:PATH"
```

## 4. Build an interface

Choose the interface you want to use. You can build more than one.

**CLI**

```sh
cargo install --path crates/cli --locked
brimp doctor
```

Expect JSON with `javascriptCore`, `lexbor`, `libcurlImpersonate`, and `crypto`
set to `ok`. Keep the SDK environment set when running the executable.

**Python**

Create and activate a virtual environment:

```sh
python3 -m venv .venv
. .venv/bin/activate
```

On Windows PowerShell, use `python -m venv .venv` and activate it with
`.venv\Scripts\Activate.ps1`. Then install the binding into that environment:

```sh
python -m pip install 'maturin>=1.9,<2'
maturin develop --locked --manifest-path bindings/python/Cargo.toml
python bindings/prepare_local.py --only python
```

**Node.js**

Build the native extension, copy it into its development package, and install
the local package in your application:

```sh
cargo build --locked -p brimp-node
python bindings/prepare_local.py --only node
npm install ./bindings/node
```

The command above uses `python`; use `python3` if that is the name of your Python
executable. In an application outside the checkout, give `npm install` the full
path to `bindings/node`.

For contributor build checks, see [Development](/development/). For creating
self-contained wheels and npm packages, see [Package native bindings](/development/packaging/).
