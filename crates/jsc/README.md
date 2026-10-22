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
