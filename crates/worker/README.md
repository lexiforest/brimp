# brimp-lite-worker

The `brimp-lite-worker` package contains the worker executable and its Chrome DevTools Protocol
subset. It reads length-prefixed CDP JSON from a local stream inherited from the
Brimp daemon. The daemon owns the public HTTP discovery and WebSocket boundary.

See [SUPPORT.md](SUPPORT.md) for the exact tested method subset.

`serve_framed` accepts any Tokio asynchronous byte stream. The
`lite-worker` executable supplies its inherited Unix socket; a Windows
host can supply a named pipe without changing protocol dispatch.

The package owns the entire lite engine. Its `runtime`, `web_apis`, `dom`,
`network`, `jsc`, and `persona` modules retain their internal responsibilities.
The build script configures both JavaScriptCore and curl-impersonate.

Browser JavaScript lives under `js/`, grouped into `web_apis/`, `runtime/`,
and `cdp/`. Rust embeds these files at compile time with `include_str!`.

`Brimp.extract` performs extraction on the page owner thread and returns a
`brimp_protocol::ExtractedDocument`. `Brimp.configure` validates persona JSON
inside the worker before creating pages.

## Native libraries

The build script expects `BRIMP_JSC_LIB_DIR` to contain:

- `JavaScriptCore.framework/JavaScriptCore` on macOS;
- `JavaScriptCore.lib` on Windows; or
- `libJavaScriptCore.so` on Linux.

It defaults to the adjacent WebKit release build used for local macOS
development. Override it for a packaged SDK:

```sh
BRIMP_JSC_LIB_DIR=/path/to/jsc-sdk/lib cargo check -p brimp-lite-worker
```

The build script also links curl-impersonate dynamically. It searches
`/usr/local/lib` by default; set `BRIMP_CURL_LIB_DIR` to another directory
containing `libcurl-impersonate.dylib` on macOS,
`libcurl-impersonate_imp.lib` on Windows, or `libcurl-impersonate.so` on Linux.

## runtime module

The public browser orchestration layer for Brimp.

Each `Page` owns one JavaScriptCore runtime, one canonical Blitz document, its
viewport, wrapper bindings, task queues, and resource-loading state. The module
provides static content, navigation, classic script scheduling, parser pausing,
events, timers, Promise-based fetch, layout reads, and CPU screenshots.

Script classification, scheduling, and module loading live in
`src/runtime/page/scripts.rs`, with the SWC compiler in
`src/runtime/page/scripts/compiler.rs`. The HTML parser remains in `dom`;
the JavaScript module loader is embedded from `js/runtime/module_loader.js`.

### Static-page example

```rust
use brimp_lite_worker::runtime::{Browser, PageOptions};

let browser = Browser::new()?;
let mut page = browser.new_local_page(
    PageOptions::builder().viewport(1280, 720).build(),
)?;
page.set_content("<div id='box' style='width: 100px'>Hello</div>")?;
page.eval("document.querySelector('#box').style.width = '200px'")?;
assert_eq!(
    page.eval("document.querySelector('#box').getBoundingClientRect().width")?
        .to_number()?,
    200.0,
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Run the static HTML/JavaScript/layout/screenshot end-to-end test with:

```sh
cargo test -p brimp-lite-worker --test runtime_mvp1
```

For network navigation, call `Page::goto().await`, then `wait_for_load().await`.
Use `Browser::with_resource_loader` to inject a deterministic or custom
`ResourceLoader`.

`Browser::new_page` returns a `PageHandle` backed by a dedicated page thread.
`Browser::new_local_page` returns a directly owned `Page` on the calling thread.
Both paths share browser resources and apply the same page setup. Local pages
are caller-owned and remain usable after the browser is closed; browser closure
shuts down managed page threads and rejects creation through either path.

### Page handles

`Browser` and `PageHandle` are the canonical interface boundary.
Each automation page owns a page/JSC runtime on one dedicated owner thread;
commands and completions cross that boundary as messages. `navigate` completes
at Brimp's `Complete` lifecycle state: the main response, parser-blocking and
deferred classic scripts, and currently discovered Blitz resources have
finished and the document has been installed. Its explicit timeout cancels the
in-flight navigation future.

`evaluate` returns JSON-compatible structured values. Undefined values,
functions, symbols, bigint values, cycles, and other values rejected by
`JSON.stringify` produce `AutomationError::Unsupported`; JavaScript exceptions
remain distinct. Screenshot bytes never pass through text conversion. Closing
a page or browser is idempotent, closes child pages, joins owner threads, and
causes later operations to return `AutomationError::Closed`.

## dom module

The canonical Blitz-backed document boundary for Brimp.

`BrowserDocument` owns the page's sole DOM tree and exposes focused operations
for traversal, selectors, mutation, style inspection, layout geometry, and
viewport updates. It does not maintain a parallel wrapper tree.

`HtmlParserSession` adds incremental html5ever parsing on the same Blitz tree.
It yields at parser-inserted scripts so the browser runtime can pause, execute
JavaScript, and resume without losing DOM mutations.

### Example

```rust
use brimp_lite_worker::dom::BrowserDocument;

let document = BrowserDocument::parse(
    "<!doctype html><html><body><div id='hello'>Hello</div></body></html>",
);
let node_id = document.get_element_by_id("hello").unwrap();
assert_eq!(document.node(node_id).unwrap().text_content(), "Hello");
```

Run the parser example with:

```sh
cargo run -p brimp-lite-worker --example dom_parse_html
```

## jsc module

RAII-oriented Rust wrappers around the public JavaScriptCore C API declared in the private `ffi` module.

The module provides JavaScript evaluation, value conversion, exception handling,
protected object handles, native callbacks, forced garbage collection, and
deferred Promise settlement. `JsRuntime` is deliberately owner-thread-bound and
is neither `Send` nor `Sync`.

`JsValue` and `JsObject` borrow their runtime. Owned `ProtectedJsObject` handles
and cloneable `JsObjectIdentity` tokens retain their JSC context and protect the
object from garbage collection until dropped. Identity tokens are rooted handles,
so keeping a token also keeps its object alive. Runtime shutdown removes native
callbacks even when owned handles keep the underlying context allocated.

### Example

```rust
use brimp_lite_worker::jsc::JsRuntime;

let runtime = JsRuntime::new()?;
let value = runtime.eval("1 + 2")?;
assert_eq!(value.to_number()?, 3.0);
# Ok::<(), brimp_lite_worker::jsc::JsException>(())
```

Run the included examples with:

```sh
cargo run -p brimp-lite-worker --example jsc_eval_js
cargo run -p brimp-lite-worker --example jsc_console
```

The worker build script owns native linking; see [Native libraries](#native-libraries).

Browser pages layer `web_apis` and `runtime` over this module. To execute
source against an installed page rather than a bare JavaScript global object,
use `brimp_lite_worker::runtime::Page::eval`; it returns a lifetime-bound `JsValue`, performs a
microtask checkpoint, and starts Fetch work queued by the script. The full
ownership and evaluation paths are documented at
[JavaScript runtime architecture](../../docs/src/content/docs/architecture/javascript-runtime.md).

## network module

Transport-neutral resource loading for Brimp.

The `ResourceLoader` trait is the single boundary used for navigation, CSS,
scripts, images, fonts, and JavaScript `fetch()`. Tests and embedders can provide
their own loader without coupling DOM or binding code to an HTTP client.

`CurlResourceLoader` is the default implementation. It owns a bounded,
long-lived libcurl-impersonate multi executor and pools reset easy handles.
Loader clones share that executor; transfers do not create request threads or
enter JavaScriptCore.

The default `chrome136` transport profile does not install curl's HTTP header
set. The resolved browsing-context persona supplies User-Agent and language
headers in insertion order, keeping transport and JavaScript identity aligned.
Curl's cookie engine and redirect
following are disabled: `runtime` owns cookies and applies every redirect
hop. Configuration is immutable for a loader's lifetime, and another loader is
required for a different profile, proxy, timeout, queue bound, or body limit.

### Core API

```rust
use brimp_lite_worker::network::{ResourceLoader, ResourceRequest};

async fn load(loader: &dyn ResourceLoader) -> Result<Vec<u8>, brimp_lite_worker::network::NetworkError> {
    let response = loader
        .fetch(ResourceRequest::get("https://example.com/"))
        .await?;
    Ok(response.body)
}
```

## web_apis module

Manual JavaScriptCore bindings for Brimp's browser-facing APIs.

The binding runtime maps stable JavaScript wrappers to Blitz node identifiers
and implements the initial DOM, CSSOM, event, timer, fetch, cookie, `Location`,
`URL`, `URLSearchParams`, and `Navigator` surfaces. Every DOM and layout
operation reads or mutates the canonical `BrowserDocument` directly.

Key internal components include:

- `BindingRuntime`, which installs and resets page globals and prototypes;
- `WrapperCache`, which preserves JavaScript object identity for native nodes;
- `TimerQueue` and `FetchQueue`, which hand work back to the page owner thread;
- `BrowsingContext`, which stores URL and cookie state.

The core browser JavaScript is split by API domain under `js/web_apis/runtime/` and
concatenated into one dependency-ordered evaluation unit. Canvas, worker, and streaming-networking scripts are installed on every page.
Persistent-storage scripts are installed when a storage path is configured. Native dispatch is split along the
same subsystem boundaries.

This module is primarily an implementation layer for `runtime`. Applications
should normally construct a `brimp_lite_worker::runtime::Browser` and interact with a `Page`
instead of installing bindings directly.

## persona module

`persona` owns the browser configuration shared by transport and JavaScript
bindings. [`src/persona/example.json`](src/persona/example.json) is the complete
configuration and the source of `PersonaConfig::default()`.

The supported top-level fields are `schema_version`, `network_profile`,
`base_headers`, `viewport`, `screen`, `window`, `navigator`, `canvas`, `locale`,
`languages`, and `timezone`. Nested fields are limited to those in the example.
JSON files must supply every field; unknown fields are rejected. There are no
presets, separate identity overrides, feature maps, or configurable seeds.

```rust
use brimp_lite_worker::persona::PersonaConfig;

let mut config = PersonaConfig::default();
config.locale = "fr-CA".into();
config.languages = vec!["fr-CA".into(), "fr".into()];
config.base_headers.accept_language = "fr-CA,fr;q=0.9".into();
config.validate()?;
# Ok::<(), brimp_lite_worker::persona::PersonaConfigError>(())
```

`network_profile` selects the curl-impersonate profile, validated by the native
transport. `base_headers` supplies request headers and JavaScript user-agent
identity, including client hints. `navigator.appVersion` is derived from the
user agent; high-entropy UA values come from the corresponding headers. Locale
and languages set `navigator.language` and `navigator.languages`; they do not
rewrite the explicitly configured `accept_language` header. Empty client-hint
headers are omitted, and an empty `sec_ch_ua` hides `navigator.userAgentData`.

Viewport, screen, window, and navigator settings are applied to pages. PDF plugin
entries follow `pdf_viewer_enabled`. Navigator flags control permissions,
Bluetooth, media devices, and OffscreenCanvas exposure; canvas is installed on every page, while persistent storage requires a configured path.
Media-device enumeration is empty because no physical devices are connected.

The retained `canvas.noise_enabled`, `canvas.noise_amplitude`, and `timezone`
fields are stored but are not yet applied to canvas pixels or JavaScriptCore's
clock. Locale does not override native `Intl` defaults. This configuration does
not claim engine-level fingerprint emulation.

## Testing

Run all worker suites with `cargo test -p brimp-lite-worker`. Individual suites
use module-prefixed names, for example `--test runtime_screenshot`.

Network integration tests bind loopback listeners, so the test environment must
permit local networking.
