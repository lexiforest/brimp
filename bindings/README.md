# Native Python and Node APIs

Python exposes a synchronous requests-style `Session`/`Page` API. Node exposes
Promise-based operations and TypeScript declarations. Both call the independent
JSC/Lexbor/curl runtime in process. Python releases the GIL during native waits;
Node runs native waits away from its event loop. Separate pages can execute
concurrently. A session owns its pages, cookie jar, request defaults, and ordered
plugin descriptors. Close it explicitly.

## Build

Install matching JavaScriptCore and curl-impersonate SDKs; set `BRIMP_JSC_LIB_DIR`
and `BRIMP_CURL_LIB_DIR` if needed (see the [runtime build notes](../crates/runtime/README.md)).
On macOS, also set `DYLD_FRAMEWORK_PATH` to the JSC framework directory when
the SDK retains the system framework install name.

```sh
cargo build -p brimp-python -p brimp-node
python3 bindings/prepare_local.py
PYTHONPATH=bindings/python/python python3 -m pytest bindings/python/tests
node --test bindings/node/test/*.test.mjs
maturin build --manifest-path bindings/python/Cargo.toml
npm pack ./bindings/node
```

The commands above produce SDK-linked development packages. On macOS,
`python3 bindings/package_native.py` assembles relocatable wheel/npm archives
from those builds, bundles JSC/curl and supplied SDK notices, records hashes,
and retags the wheel with the libraries' actual minimum macOS version. See
[package assembly](PACKAGING.md). Other platforms retain the SDK installation
workflow.

The page realm includes DOM lifecycle and Range operations, FileReader, Web
Streams, structured cloning, XPath, Shadow DOM and WebCrypto. See
[core web coverage](CORE_WEB.md) for algorithm support and behavior limits.
Use [plugin discovery and backend options](PLUGINS.md#discovering-capabilities-and-choosing-storage-lifetime)
to choose additional capabilities and storage lifetimes.

## Requests, values, and DOM

See the [Python](python/README.md) and [Node](node/README.md) examples. Request verbs
return a reusable `Page`, with a response snapshot containing bytes/Buffer,
headers (including repeated fields), status, URL, redirects, request metadata,
and timing. Defaults cover headers, timeout, proxy, and Basic authentication.
Python timeouts/latency use seconds; Node uses milliseconds. Node requests and
streams accept AbortSignal. `html()` serializes the edited DOM.

`evaluate` awaits Promises while pumping timers, networking, and host completions.
Use `evaluate_handle` / `evaluateHandle` for values that remain in the realm.
Handles support function calls, properties, JSON conversion, and disposal.
Element handles add attributes, text/HTML, subtree queries, mutation, focus,
file inputs, and removal. Detachment preserves identity; navigation invalidates
old handles with `StaleHandle`. Page closure rejects operations with `Closed`.

Selector-based click, hover, type, tap, focus, fill, press, and element-to-element
drag/drop target DOM nodes. Native defaults preserve event cancellation and form
editing. There is no visibility/coordinate actionability, scrolling, geometry,
computed style, or screenshot API. Session/page identity sets user agent,
languages, and platform; document-start scripts run before document scripts.

## Events and routes

`page.on(event, handler)` returns an unsubscribe function. Events include console,
error, lifecycle, mutation, request, response, request completion/failure,
WebSocket, and EventSource. Python records use snake_case; Node uses camelCase.
Node's event names are `requestFinished`, `requestFailed`, `webSocket`, and
`eventSource`; Python uses `request_finished`, `request_failed`, `websocket`, and
`eventsource`. Event callbacks are dispatched in emission order; asynchronous
Node callbacks may finish in a different order.

`page.route(pattern, handler, stage=...)` (Node: `{stage}`) returns a removal
function. Patterns use glob matching; the last matching route wins. Stages are
`request`, `response`, and `auth`. Complete each route exactly once with
continuation/overrides, fulfillment, failure, or authentication. Exceptions or a
handler returning without a decision abort the request. Response/auth routes
buffer at most 32 MiB. `body()` returns copied bytes. Registered routes bypass
cache lookup so interception remains effective.

Handlers may use a different page. Reentry into the same page, including through
its handles, fails immediately with `Reentry`. Python callbacks run on the
session's managed dispatcher and must return synchronously; asyncio callbacks
are not supported. Node route and plugin callbacks may return Promises.

## Network controls and streams

`configure_network` / `configureNetwork` sets offline mode, latency, cache bypass,
retention limits, and optional buffered download delay. `set_extra_headers` /
`setExtraHeaders`, `clear_cache` / `clearCache`, and `response_body` / `responseBody`
are immediate controls/snapshots. The small in-memory GET cache honors max-age,
no-store/no-cache, and excludes Vary and Set-Cookie responses. Retained bodies
are bounded separately (32 MiB total, 8 MiB per body by default).

`page.stream(method, url, ...)` returns response headers followed by a bounded
body consumer without replacing the page's DOM. Python returns a context-managed
chunk iterator. Node returns `{status, url, headers, body, cancel}`, where `body`
is a Readable with Buffer chunks. Native buffering is 256 KiB plus transport
buffers; Node adds its readable buffer. Intercepted responses retain the separate
32 MiB buffering limit. Close/cancel abandoned streams. Navigation and page/session
closure cancel pending streams. Buffered latency controls do not throttle bytes
on the wire.

```python
with page.stream("GET", "https://example.com/data") as response:
    for chunk in response:
        consume(chunk)
```

```javascript
const response = await page.stream('GET', 'https://example.com/data', {signal});
for await (const chunk of response.body) consume(chunk);
```

See [plugins](PLUGINS.md) for optional storage, Canvas, services, extraction, and
custom host implementations.

See [diagnostics and programmable time](DEBUGGING.md) for tracing, logical clocks,
bounded task draining and live-network timing limits.
