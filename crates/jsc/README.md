# jsc

RAII-oriented Rust wrappers around the public JavaScriptCore C API exposed by
`jsc-sys`.

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

The JavaScriptCore framework setup is documented in `../jsc-sys/README.md`.

Browser pages layer `web-bindings` and `web-runtime` over this crate. To execute
source against an installed page rather than a bare JavaScript global object,
use `web_runtime::Page::eval`; it returns a lifetime-bound `JsValue`, performs a
microtask checkpoint, and starts Fetch work queued by the script. The full
ownership and evaluation paths are documented at
`docs/src/content/docs/architecture/javascript-runtime.md`.
