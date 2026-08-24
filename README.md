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

## Prerequisites

The current JSC binding targets macOS and dynamically links a JSCOnly build at:

```text
../WebKit/WebKitBuild/Release/JavaScriptCore.framework
```

Set `BRIMP_JSC_FRAMEWORK_DIR` to use a different framework directory. The
network crate uses the adjacent `bimp-net` libcurl-impersonate wrapper declared
in `crates/network/Cargo.toml`.

## Run

```sh
cargo run -p web-runtime --example mvp1
```

This evaluates JavaScript against a Blitz document and writes `out.png` through
the Vello CPU renderer.

## Verify

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The network integration test binds a loopback listener and therefore needs an
environment that permits local networking.

See `PLAN.md` for the architecture and `TODO.md` for acceptance evidence and
scope status.
