---
title: Sessions, pages and requests
description: Fetch a page, reuse cookies, wait for JavaScript, and intercept requests.
---

Use a session for work that shares cookies and request settings. Create a page
for each document you want to keep open. Close the session when you finish;
this closes its pages and releases their resources.

The examples use the [local tutorial server](/quick-start/#1-start-the-example-site).
Python calls are synchronous; Node.js calls return Promises. Request timeouts use
seconds in Python and milliseconds in Node.js.

## Fetch a page and read the result

```python
import brimp

with brimp.Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    print(page.status_code)
    print("Waiting for JavaScript" in page.text)
    print(page.evaluate("document.querySelector('h1').textContent"))
```

This prints `200`, `True`, and `Hello from JavaScript`. The original HTTP response
contains the waiting message; the page's script changes the DOM.

| You need | Read |
| --- | --- |
| Original response text or JSON | `page.text` or `page.json()` |
| Status, headers, URL or redirects | `page.status_code`, `page.headers`, `page.url`, `page.history` (camelCase in Node.js) |
| Current HTML after scripts or edits | `page.html()`; await it in Node.js |
| A value computed inside the page | `page.evaluate(...)`; await it in Node.js |

The returned `Page` provides both HTTP metadata and live document operations.
There is no separate public response object. The most recent successful request
replaces the HTTP metadata; DOM edits leave the original response body unchanged.
Use `page.request_info` in Python or `page.requestInfo` in Node.js for the original
request record; `page.request(...)` performs another navigation.

HTTP error responses still return a page with its body and DOM available. Call
`page.raise_for_status()` in Python or `page.raiseForStatus()` in Node.js to raise
`HTTPError` for 4xx/5xx responses. Other HTTP statuses return normally. The error
includes `page` plus the failed status and URL (`status_code` / `statusCode`, and
`url`), which remain available even if the page navigates again.

Blank pages and pages populated by `load_html()` / `loadHtml()` have no HTTP
response. Reading their HTTP properties or checking status raises `InvalidInput`
in Python or `RuntimeError` with code `invalid_input` in Node.js. Their DOM and
JavaScript operations are still available. Streaming downloads return a separate
stream and leave the page's navigation metadata unchanged.

See the [Python](/reference/python/) and
[Node.js](/reference/node/) references for all response properties.

## Reuse a page or open another one

`session.get()` creates a new page. `page.get()` navigates an existing page to a
new document. Both use the session's cookie jar:

```python
import brimp

with brimp.Session() as session:
    session.set_cookie("http://127.0.0.1:8765/", "theme", "dark")
    page = session.get("http://127.0.0.1:8765/")
    page.get("http://127.0.0.1:8765/echo")
    print(page.json()["cookie"])
    other = session.get("http://127.0.0.1:8765/article")
    print(other.evaluate("document.querySelector('h1').textContent"))
```

The cookie header includes `theme=dark`. The second page prints
`Hello from JavaScript`. Pages have separate JavaScript globals and documents;
sharing a session shares cookies and connections, not page variables.

Navigation replaces the page's JavaScript environment, called its **realm**.
Handles from the previous document then raise `StaleHandle`. Use
[handles](/guides/values/) when you need to keep a DOM node or JavaScript object
between calls, and obtain fresh handles after navigation.

## Wait for a JavaScript result

Return a Promise from `evaluate()` to wait for it. Timers, network requests and
host callbacks can complete while Brimp waits:

```python
import brimp

with brimp.Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    result = page.evaluate("fetch('/echo').then(response => response.json())")
    print(result["method"])
```

This prints `GET`. Starting background work without returning its Promise does
not wait for that work to finish. Return the result you need, or check the
application's completion condition before reading it.

## Attach a route before navigation

Create a blank page when you need to install listeners or routes before scripts
run. This Node.js example supplies an API response without contacting the server
for that request:

```js
import { createSession } from '@brimp/brimp';

const session = await createSession();
try {
  const page = await session.newPage();
  page.route('http://127.0.0.1:8765/api/*', route => route.fulfill({
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({source: 'route'}),
  }));
  await page.get('http://127.0.0.1:8765/');
  console.log(await page.evaluate('fetch("/api/status").then(response => response.json())'));
} finally {
  await session.close();
}
```

Expect `{ source: 'route' }`. Patterns use glob matching; the last matching route
wins. Complete each route exactly once by continuing, fulfilling, failing, or
authenticating it. Returning without a decision or throwing aborts the request.
`page.route()` returns a function that removes the route.

The default route stage is `request`. Use `response` to change a response or
`auth` to supply credentials. Response and authentication routes buffer bodies,
so use [streaming](/examples/fetch/#3-download-without-replacing-the-dom) for
large downloads that do not need interception.

## Observe events and handle callback errors

`page.on(event, handler)` subscribes to console messages, errors, lifecycle
changes, DOM mutations and network activity. It returns an unsubscribe function.
Attach listeners before starting the operation you want to observe.

Python event names use snake_case, such as `request_finished`, `request_failed`,
`websocket`, and `eventsource`. Node.js uses `requestFinished`, `requestFailed`,
`webSocket`, and `eventSource`. Callbacks start in emission order; asynchronous
Node.js callbacks can finish in a different order.

A callback cannot evaluate, navigate, or close its own page: those calls raise
`Reentry` because the page is already waiting for the callback. It can use
another page or operate on handles supplied to it. A callback's handle operation
cannot call another host handler recursively. Python callbacks must return
synchronously; Node.js route and plugin callbacks may return Promises.

For complete examples, see [interception](/examples/fetch/#2-supply-an-api-response-with-a-route)
and [debugging](/examples/debugging/).

## Choose input, network controls, or streaming

- Use `fill`, `click`, `press`, and related methods to interact with DOM elements.
  Actions use nodes directly; they do not check visibility or coordinates.
  See the [Python](/examples/python/#5-fill-the-form-and-save-the-edited-html)
  and [Node.js](/examples/node/#2-work-with-dom-handles) tutorials.
- Use `configure_network()` / `configureNetwork()` for offline mode, latency,
  cache bypass, and response retention. `set_extra_headers()` / `setExtraHeaders()`
  changes subsequent request headers. `clear_cache()` / `clearCache()` clears
  cached responses; `response_body()` / `responseBody()` reads a retained body.
- Use `page.stream()` to download a body in chunks without replacing the DOM.
  Close or cancel a stream you stop reading. See [streaming examples](/examples/fetch/#3-download-without-replacing-the-dom).
- Use [identity options and profiles](/guides/profiles/) to change browser values,
  and [plugins](/guides/plugins/) to add extraction or host callbacks.

<details>
<summary>Concurrency, buffering and cache limits</summary>

Separate pages can execute concurrently. Python releases the GIL during native
waits; Node.js performs them away from its event loop.

Response/authentication routes buffer at most 32 MiB. Registering a route bypasses
cache lookup so it still sees matching requests. The in-memory GET cache honors
max-age, no-store and no-cache; it excludes Vary and Set-Cookie responses.
Retained response bodies have separate limits: 32 MiB total and 8 MiB per body
by default.

Native streaming buffering is 256 KiB plus transport buffers. Node.js adds its
Readable buffer. Navigation and page/session closure cancel pending streams.
Buffered download delays do not throttle bytes on the wire.

</details>
