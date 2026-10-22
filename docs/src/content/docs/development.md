---
title: Development
---

The workspace contains `brimp-lexbor` (safe DOM arena and native parser),
`brimp-jsc` (engine ownership and callbacks), `brimp-net` (transport), and
`brimp-runtime` (browser DOM identity, Web APIs, sessions and plugins). Each lives
in the matching `crates/` directory. PyO3/napi-rs bindings in `bindings/` depend
on the runtime; the runtime depends on the three independent lower crates.
The runtime dynamically links JSC and curl-impersonate.

Lexbor builds and tests independently, with no external native SDKs:

```sh
cargo test -p brimp-lexbor
```

JSC can be tested with only its engine SDK using `cargo test -p brimp-jsc`.
Raw FFI stays private in both binding crates.

## Native prerequisites

`BRIMP_JSC_LIB_DIR` contains `JavaScriptCore.framework/JavaScriptCore` on macOS,
`JavaScriptCore.lib` on Windows, or `libJavaScriptCore.so` on Linux.
`BRIMP_CURL_LIB_DIR` contains the corresponding curl-impersonate library.
Core WebCrypto builds OpenSSL from Cargo-pinned source through rust-openssl
(vendored feature). A C compiler, make and Perl are required; consumers do not
need an OpenSSL shared library. Its license notices are packaged with bindings.
The macOS development defaults are the adjacent WebKit release directory and
`/usr/local/lib`. Lexbor 3.0.0 is vendored with license, archive hash, and documented
local patches. No native rendering, fonts, persistent-storage, or worker engines
are linked into the DOM runtime.

For a local macOS WebKit build, export
`DYLD_FRAMEWORK_PATH="$PWD/../WebKit/WebKitBuild/Release"` when running tests or
bindings. This selects the rebuilt JSC even when its install name points to the
system framework. Downloaded SDK setup exports the corresponding directory.

```sh
cargo build -p brimp-python -p brimp-node
python3 bindings/prepare_local.py
cargo test -p brimp-runtime
PYTHONPATH=bindings/python/python python3 -m pytest bindings/python/tests
node --test bindings/node/test/*.test.mjs
python3 scripts/test_lexbor_boundary.py
cargo clippy --workspace --all-targets -- -D warnings
```

Benchmarks in `benchmark/native` compare parse/select/mutate and binding workflows
on the same host/debug profile, with elapsed time and peak RSS. Focused regression
tests cover the supported DOM, networking, and plugin contract.


The standalone `brimp-net` transport can be tested without JSC:

```sh
cargo test -p brimp-net
cargo clippy -p brimp-net --all-targets -- -D warnings
```

Its tests use loopback HTTP fixtures. The runtime uses `brimp-net` for HTTP and WebSocket transport, with browser
resource policy kept in the runtime. The `network_benchmark` example compares
the runtime adapter with direct transport calls. The comparison measurements and
reproduction commands are in `benchmark/network/README.md`.
