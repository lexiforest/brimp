# brimp-jsc

JavaScriptCore ownership, evaluation, values, native callbacks, exceptions,
protected object handles and deferred promises. Raw FFI is private. Engine
objects remain on their owner thread; browser APIs and page policy live in
brimp-runtime. This crate has no DOM or network dependency.

Set `BRIMP_JSC_LIB_DIR` to the directory containing the JavaScriptCore framework
(macOS), import library (Windows), or shared library (Linux). The default is the
adjacent `WebKit/WebKitBuild/Release` directory. Locally built macOS frameworks
may also require `DYLD_FRAMEWORK_PATH` to select the rebuilt SDK at runtime.

```sh
cargo test -p brimp-jsc
cargo run -p brimp-jsc --example eval_js
cargo run -p brimp-jsc --example console
```

No curl, Lexbor, Node or Python SDK is required.

Embedders must call `JsRuntime::pump_jobs()` on the owner thread between host
operations. This services asynchronous engine work, including WebAssembly
compilation. It never waits for an event. Brimp's page scheduler calls it during
Promise waits and idle processing with a maximum 10 ms wait between checks.

On macOS this uses CoreFoundation's current-thread run loop. The pinned Linux
and Windows SDKs export `WTF::RunLoop::cycle(unsigned)`, which provides a
nonblocking iteration; that private scalar ABI is declared in `ffi.rs` alongside
the existing pinned SDK APIs. The engine CI job verifies linking and asynchronous
WASM execution on those targets. SDK upgrades must preserve or update this
boundary; there is no silent fallback to synchronous WASM compilation.
