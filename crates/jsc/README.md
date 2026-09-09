# jsc

RAII-oriented Rust wrappers around the public JavaScriptCore C API declared in the private `sys` module.

The crate provides JavaScript evaluation, value conversion, exception handling,
protected object handles, native callbacks, forced garbage collection, and
deferred Promise settlement. `JsRuntime` is deliberately owner-thread-bound and
is neither `Send` nor `Sync`.

## Example

```rust
use jsc::JsRuntime;

let runtime = JsRuntime::new()?;
let value = runtime.eval("1 + 2")?;
assert_eq!(value.to_number()?, 3.0);
# Ok::<(), jsc::JsException>(())
```

Run the included examples with:

```sh
cargo run -p jsc --example eval_js
cargo run -p jsc --example console
```

The crate owns native linking; see the linking instructions below.

Browser pages layer `web-bindings` and `web-runtime` over this crate. To execute
source against an installed page rather than a bare JavaScript global object,
use `web_runtime::Page::eval`; it returns a lifetime-bound `JsValue`, performs a
microtask checkpoint, and starts Fetch work queued by the script. The full
ownership and evaluation paths are documented at
`docs/src/content/docs/architecture/javascript-runtime.md`.

## Linking

The build script expects `BRIMP_JSC_LIB_DIR` to contain:

- `JavaScriptCore.framework/JavaScriptCore` on macOS;
- `JavaScriptCore.lib` on Windows; or
- `libJavaScriptCore.so` on Linux.

It defaults to the adjacent WebKit release build used for local macOS
development. Override it for a packaged SDK:

```sh
BRIMP_JSC_LIB_DIR=/path/to/jsc-sdk/lib cargo check -p jsc
```
