---
title: JavaScriptCore integration
description: How Brimp embeds JavaScriptCore, installs Web APIs, and evaluates scripts.
---

Brimp embeds WebKit's JavaScriptCore through its public C API. JavaScriptCore
provides the language engine and garbage collector; Brimp supplies the browser
objects, DOM, networking, event loop integration, and optional subsystems.

## Integration layers

```text
Page and AutomationPage
        │
        ▼
web-runtime: owner thread, lifecycle, tasks, navigation
        │
        ▼
web-bindings: Window/DOM/Web API JavaScript plus native dispatch
        │
        ▼
jsc: RAII values, protected objects, callbacks, exceptions, promises
        │
        ▼
jsc-sys: unsafe JavaScriptCore C API and platform linkage
```

`jsc-sys` is the unsafe ABI boundary. It declares opaque JavaScriptCore handles
and links the platform library selected by `BRIMP_JSC_LIB_DIR`. The `jsc` crate
wraps those handles with Rust lifetimes, exception conversion, garbage-collector
protection, native callbacks, and deferred Promise settlement.

`web-bindings` installs browser-facing JavaScript classes and one native host
entry point. JavaScript wrappers retain normal Web-IDL-shaped objects while
native operations dispatch to Rust-owned DOM, Canvas, WebGL, WebGPU, WebAudio,
storage, and networking state. DOM wrappers cache native node identities so the
same native node returns the same JavaScript object.

The shared JavaScript bootstrap is assembled from dependency-ordered files in
`crates/web-bindings/src/runtime/` and evaluated as one script. This keeps one
lexical scope and deterministic installation order. Optional subsystem scripts
are evaluated only when their page option is enabled.

## Thread ownership

A JavaScriptCore context is owner-thread-bound and is neither `Send` nor `Sync`.
The low-level `Page` must remain on its creating thread. `AutomationPage`
provides the thread-safe command boundary used by the CLI and language bindings:
it owns the low-level page on a dedicated thread and exchanges typed commands
and results through channels.

Network and worker callbacks never call JavaScriptCore from arbitrary threads.
They enqueue work for the page owner thread, which performs JavaScript execution,
microtask checkpoints, timers, and event dispatch.

## Evaluate directly on a page

For an embedded Rust page, call `Page::eval()`:

```rust
use web_runtime::{Browser, PageOptions};

let browser = Browser::new()?;
let mut page = browser.new_page(PageOptions::default())?;

page.set_content("<title>Direct evaluation</title>")?;

let title = page.eval("document.title")?.to_string()?;
let answer = page.eval("6 * 7")?.to_number()?;

assert_eq!(title, "Direct evaluation");
assert_eq!(answer, 42.0);

# Ok::<(), Box<dyn std::error::Error>>(())
```

`Page::eval()` returns a lifetime-bound `jsc::JsValue`. It performs a
JavaScriptCore microtask checkpoint and starts Fetch operations queued by the
script before returning. It does not JSON-serialize the result. Convert the
value while the page/runtime borrow is valid using `to_number()`, `to_string()`,
or `to_object()`.

Use `AutomationPage::evaluate()` when the caller cannot live on the JavaScript
owner thread:

```rust
use web_runtime::{AutomationBrowser, PageOptions};

let browser = AutomationBrowser::new()?;
let page = browser.new_page(PageOptions::default())?;
let value = page.evaluate("({ title: document.title, answer: 6 * 7 })")?;

println!("{value}");
page.close();
browser.close();

# Ok::<(), Box<dyn std::error::Error>>(())
```

The automation path returns `serde_json::Value`. Functions, symbols, `BigInt`,
cycles, top-level `undefined`, and other non-JSON results return
`AutomationError::Unsupported`. JavaScript exceptions remain a distinct
`AutomationError::JavaScript` failure.

Python `Session.evaluate()`, Node `page.evaluate()`, CLI `brimp eval`, and CDP
`Runtime.evaluate` all delegate to this owner-thread machinery. CDP additionally
supports page-owned remote object handles through `Runtime.callFunctionOn`,
`Runtime.getProperties`, and the release methods.

## Native-looking Web APIs

Browser API constructors and methods are implemented partly in JavaScript and
partly in Rust. During bootstrap, Brimp records browser-provided functions and
patches `Function.prototype.toString` so those functions expose native-function
syntax such as `function querySelector() { [native code] }`. Ordinary page
functions still return their actual source. Persona and integration tests check
this distinction together with Web-IDL descriptors and prototype tags.

## Linking JavaScriptCore

Source builds set `BRIMP_JSC_LIB_DIR` to a directory containing:

- `JavaScriptCore.framework/JavaScriptCore` on macOS;
- `JavaScriptCore.lib` on Windows; or
- `libJavaScriptCore.so` on Linux.

Packaged bindings bundle the expected JavaScriptCore runtime. See the
[installation guide](/install/) and repository `NATIVE.md` for platform layouts.
