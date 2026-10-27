---
title: Sessions, pages and requests
description: Navigate pages, read responses, handle events, intercept requests and stream bodies.
---

Python exposes a synchronous requests-style `Session`/`Page` API. Node exposes
Promise-based operations and TypeScript declarations. Both call the independent
JSC/Lexbor/curl runtime in process. Python releases the GIL during native waits;
Node runs native waits away from its event loop. Separate pages can execute
concurrently. A session owns its pages, cookie jar, request defaults, and ordered
plugin descriptors. Close it explicitly.

## Requests, values, and DOM

See the [Python](https://github.com/lexiforest/brimp/blob/main/bindings/python/README.md) and [Node](https://github.com/lexiforest/brimp/blob/main/bindings/node/README.md) examples. Request verbs
return a reusable `Page`, with a response snapshot containing bytes/Buffer,
headers (including repeated fields), status, URL, redirects, request metadata,
and timing. Defaults cover headers, timeout, proxy, and Basic authentication.
Python timeouts/latency use seconds; Node uses milliseconds. Node requests and
streams accept AbortSignal. `html()` serializes the edited DOM.

`evaluate` awaits Promises while pumping timers, networking, and host completions.
Use `evaluate_handle` / `evaluateHandle` for values that remain in the realm.
Handles support function invocation with an explicit receiver, nested handle arguments,
property reads/writes/deletion, rich value copying, explicit JSON conversion, and disposal.
See [value bridging](/guides/values/) for the Python/Node conversion contract.
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

Handlers may use a different page and operate on references from their own realm.
Same-page lifecycle/evaluation reentry fails with `Reentry`; a callback handle
operation cannot recursively call another host handler. Python callbacks run on the
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

See [plugins](/guides/plugins/) for the default browser subsystems, core storage
lifetimes, and explicit extraction and host-storage integrations.
