# Brimp

Brimp is a lightweight, headless browser runtime written in Rust. It combines
JavaScriptCore with Blitz's DOM, Stylo, Taffy, Parley, and CPU paint pipeline.
The Blitz tree is the sole DOM representation exposed to JavaScript.

The implemented runtime supports:

- static HTML, JavaScript DOM mutation, style/layout queries, and CPU PNG screenshots;
- HTTP(S) navigation through a transport-neutral `ResourceLoader`;
- linked CSS, common raster images, classic inline/external scripts, `defer`, and `async`;
- parser pausing around blocking scripts;
- events, timers, microtasks, Promise-based `fetch`, cookies, `Location`, and `Navigator`.

The canonical owner-thread automation API is exposed through:

- the `brimp` CLI for evaluation and screenshots;
- asynchronous in-process Python and Node native bindings; and
- a bounded loopback CDP server for the checked Puppeteer workflow.

All four interfaces delegate navigation, JavaScript, lifecycle, and screenshots
to `web-runtime`; none contains a second browser implementation.

## Prerequisites

The current JSC binding targets macOS and dynamically links a JSCOnly build at:

```text
../WebKit/WebKitBuild/Release/JavaScriptCore.framework
```

Set `BRIMP_JSC_FRAMEWORK_DIR` to use a different framework directory. The
Brimp-owned network transport links `libcurl-impersonate` from `/usr/local/lib`;
set `BRIMP_CURL_LIB_DIR` to use another location.

## Run

```sh
cargo run -p web-runtime --example mvp1
```

This evaluates JavaScript against a Blitz document and writes `out.png` through
the Vello CPU renderer.

Other entry points are:

```sh
cargo run -p brimp-cli -- doctor
cargo run -p brimp-benchmark --release -- --samples 20
uv run --project benchmark/performance --no-sync python benchmark/performance/bench.py
uv run --project benchmark/performance --no-sync python benchmark/memory/memory.py
cargo run -p brimp-cdp -- --bind 127.0.0.1:9222
```

The Python and Node bindings are native extensions loaded into their host
processes. They are not CDP clients and do not start a server. CDP is the only
server/client interface in this workspace.

## Verify

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
python3 -m unittest discover -s benchmark/performance -p 'test_*.py' -v
python3 -m unittest discover -s benchmark/memory -p 'test_*.py' -v
./bindings/package-test.sh
./crates/cdp/puppeteer-test.sh
```

The network integration test binds a loopback listener and therefore needs an
environment that permits local networking.

Current release artifacts target macOS arm64. Python and Node packages bundle
the arm64 libcurl-impersonate dylib and use macOS system frameworks at runtime;
source builds use the configurable native discovery paths above. See each
interface's `SUPPORT.md` for its exact tested surface.

See `PLAN.md` for the architecture and `TODO.md` for acceptance evidence and
scope status.
