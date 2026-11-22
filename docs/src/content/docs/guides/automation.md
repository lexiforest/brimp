---
title: Sessions, pages and requests
description: Fetch a page, reuse cookies, wait for JavaScript, and intercept requests.
---

Use a session for work that shares cookies and request settings. Create a page
for each document you want to keep open. Close the session when you finish;
this closes its pages and releases their resources.

The examples use the [local tutorial server](/quick-start/#1-start-the-example-site).
Python's `Session` is synchronous; `AsyncSession` supports asyncio. Node.js calls
return Promises. Request timeouts use seconds (including fractions) in Python and milliseconds in Node.js.
Python accepts `None`/`0` to disable the deadline. Timeout tuples are unsupported. Omitting the argument inherits the session
setting. See [Python timeouts](/api/python/#timeouts).

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

## Make a one-off Python request

Module-level helpers return the same live `brimp.Page` as a session request:

```python
import brimp

with brimp.get("http://127.0.0.1:8765/") as page:
    page.raise_for_status()
    print(page.evaluate("document.querySelector('h1').textContent"))
```

This prints `Hello from JavaScript`. The helper keeps a private session alive
until you close the page. Use a `with` block or call `page.close()` to release
both; the original HTTP metadata and body remain readable afterward. Each helper
call has separate cookies and connections. Use `brimp.Session` to share them or
configure plugins, profiles, or other session settings.

Available helpers are `request(method, url)`, `get`, `head`, `post`, `put`, `patch`,
`delete`, `options`, `trace`, and `query`. They accept the keyword arguments of
`Page.request`; session and page methods support the same HTTP verbs.

## Construct Python request URLs

Set a session `base_url` to resolve relative URLs and `params` for default query
values. Request-level `params` accepts a mapping or a sequence of key/value pairs:

```python
import brimp

with brimp.Session(
    base_url="http://127.0.0.1:8765/",
    params={"source": "agent"},
) as session:
    page = session.get("echo", params={"q": "browser runtime", "tag": ["a", "b"]})
    print(page.url)
```

The URL ends in `/echo?source=agent&q=browser+runtime&tag=a&tag=b`.
You can update `session.base_url` and `session.params` before subsequent requests.
These settings apply to `Page.request` and `Page.stream` too; they do not rewrite
requests made by scripts inside the page.

Query values merge in this order: the supplied URL, session params, then request
params. A single incoming value replaces a single existing value with the same
key; repeated keys append, preserving their order. Lists produce repeated query
keys, and blank values are retained. Following curl-cffi, booleans and dictionary
values use JSON encoding. `None` becomes the string `None`; it does not remove a
default parameter.

The merged URL then resolves against `base_url` using normal relative-URL rules.
A leading `/` starts at the host root; an absolute URL overrides the base. End
the base with `/` to treat its final path component as a directory. A base query
is inherited only when the relative URL has neither a path nor a replacement
query.

Automatic quoting encodes spaces and non-ASCII characters while preserving
reserved percent escapes. Pass `quote=":"` to additionally encode colons in the
path and query, or `quote=False` to skip automatic Python quoting. Explicit
`params` still undergo query encoding. The native transport continues to validate
URLs and may normalize their spelling, such as the case of percent escapes.

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

## Use Python with asyncio

Use `brimp.AsyncSession` to fetch and interact with pages without blocking the
asyncio event loop:

```python
import asyncio
import brimp

async def main():
    async with brimp.AsyncSession(
        base_url="http://127.0.0.1:8765/",
        max_clients=4,
    ) as session:
        pages = await asyncio.gather(session.get("/"), session.get("/article"))
        for page in pages:
            page.raise_for_status()
            async with page:
                heading = await page.query_selector("h1")
                async with heading:
                    print(await heading.text_content())

asyncio.run(main())
```

This prints `Hello from JavaScript` twice. Async sessions accept the same options
and request arguments as synchronous sessions. Requests return `brimp.AsyncPage`.
Await navigation, evaluation, DOM operations, handle methods and `close()`.
Buffered HTTP properties such as `status_code`, `text` and `content`, plus
`json()` and `raise_for_status()`, are synchronous. HTTP errors refer to the async
page through `error.page`.

`max_clients` defaults to 10 and limits concurrent native operations. Separate
pages can run concurrently; operations on one page run in order. A completed
request releases its concurrency slot while its page remains usable. Create and
use each session on one event loop, and close it with `async with` or
`await session.close()`.

Cancelling a task signals its native operation and waits for that call to finish
before returning `asyncio.CancelledError`. Cancelling a queued call prevents it
from starting. A cancelled `session.get()` closes its newly created page; a
cancelled operation on an existing page leaves that page open. Cancellation does
not undo JavaScript changes or restore a document interrupted during navigation.
Closing the session interrupts active work, rejects queued work and releases its
pages and executor threads. Cleanup completes even if the close task is cancelled.

`page.on()` and `page.route()` still register synchronous callbacks. Plugin
handlers also remain synchronous and run on the session's callback dispatcher.
Use `await session.route()` to register a synchronous callback across pages.
Async streaming and coroutine callbacks are not yet supported.

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

## Wait and act on a changing document

Use a locator when an element can appear, become enabled, or be replaced after
you first select it. A locator keeps selectors rather than a rooted DOM node.
Each operation resolves the current document again. Single-element operations
reject multiple matches; narrow the selector or use `nth(index)` explicitly.

```python
import brimp

with brimp.Session() as session:
    page = session.get("http://127.0.0.1:8765/", wait_until="domcontentloaded")
    page.wait_for_load_state("load")
    page.wait_for_function("() => document.querySelector('h1') !== null", timeout=5)
    heading = page.locator("h1")
    heading.wait_for(state="visible", timeout=5)
    print(heading.text_content())
```

The Node equivalents are `waitUntil`, `waitForLoadState`, `waitForFunction`,
`waitForSelector`, and `locator(...).waitFor(...)`. Python durations are seconds;
Node durations are milliseconds. Omit `timeout` to inherit the session default,
or use zero to disable it. Python also accepts `None`. Node operation options
accept `signal: AbortSignal`; cancelling an asyncio task interrupts its native
wait. `evaluate` accepts the same operation timeout and cancellation controls.

Selector states are `attached`, `detached`, `visible`, and `hidden`; the default
is `visible`. Function waits repeatedly evaluate an expression or call a returned
function with the JSON-compatible `arg` option, awaiting Promises and returning
the first truthy result. Predicate exceptions propagate immediately.
`wait_for_network_idle(quiet=0.5)` / `waitForNetworkIdle({quiet: 500})` waits for a
quiet interval with no active requests. Prefer an application condition when a
page polls continuously. Draining currently due tasks is a separate operation.

Locators support `click`, `hover`, `fill`, `type_text` / `typeText`, `press`,
`text_content` / `textContent`, `get_attribute` / `getAttribute`, `count`, nested
`locator` queries, and `nth`. Input actions wait for a connected, visible,
enabled target with a stable box; text entry also checks editability, and pointer
actions check that the target receives the center-point hit. Resolution, checks,
and input dispatch occur on the page owner thread. Existing `page.click` and
handle actions remain immediate operations.

These checks use Brimp's layout engine. Unsupported layout, scrolling and paint
behavior still limit actionability; locators do not add a full browser renderer.
Hidden or covered targets time out instead of receiving a forced click.

## Subscribe before an action

Subscribe before triggering navigation or a request, so an immediate event is
not missed. Python expectations remove their listener on success, timeout,
cancellation, or an exception in the action:

```python
import brimp

with brimp.Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    with page.expect_response("http://127.0.0.1:8765/echo") as pending:
        page.evaluate("fetch('/echo').then(() => null)")
    print(pending.value["response"]["status"])

    with page.expect_navigation():
        page.evaluate("location.href = '/article'")
    print(page.current_url())
```

For `AsyncPage`, use `async with` and `await pending.value`. Generic
`expect_event(event, predicate=...)` supports the documented page events;
predicates are synchronous. `expect_navigation(wait_until="domcontentloaded")`
selects the earlier milestone; the default is `load`.

Node subscriptions begin when the wait method is called:

```js
import { createSession } from '@brimp/brimp';

const session = await createSession();
try {
  const page = await session.get('http://127.0.0.1:8765/');
  const pending = page.waitForResponse('http://127.0.0.1:8765/echo', {timeout: 5000});
  await page.evaluate("fetch('/echo').then(() => null)");
  console.log((await pending).response.status);

  await Promise.all([
    page.waitForNavigation(),
    page.evaluate("location.href = '/article'"),
  ]);
  console.log(await page.currentUrl());
} finally {
  await session.close();
}
```

`waitForEvent(event, {predicate, timeout, signal})` provides generic observation.
Response convenience waits match an exact URL; use a generic event predicate for
other matching rules. Closing the page/session rejects pending expectations.
Navigation milestones cover document/script loading; use an application condition
or network-idle wait when you also need background requests to finish.

## Frames and popups

`page.frame_locator("iframe#checkout")` / `page.frameLocator('iframe#checkout')`
returns a frame scope. Use `frame.locator(...)` to interact with its document,
`frame.evaluate(...)` to run in its JavaScript realm, and another `frame_locator`
to enter a nested frame. These selector paths resolve again after replacement.
`page.frames()` and `frame.frames()` discover the frames in the current document;
await them in Python async and Node. Generated discovery paths describe the
current DOM position; use stable selectors when the page rearranges its frames.

Opening `window.open(...)` or clicking a link with a new-window target creates a
session-owned page with shared cookies, settings and plugins:

```python
import brimp

with brimp.Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    with page.expect_popup(timeout=10) as pending:
        page.evaluate("void window.open('/article', '_blank')")
    popup = pending.value
    print(popup.current_url())
    print(popup.evaluate("document.title"))
    popup.close()
```

```js
import { createSession } from '@brimp/brimp';

const session = await createSession();
try {
  const page = await session.get('http://127.0.0.1:8765/');
  const pending = page.waitForPopup({timeout: 10000});
  await page.evaluate("void window.open('/article', '_blank')");
  const popup = await pending;
  console.log(await popup.currentUrl());
  await popup.close();
} finally {
  await session.close();
}
```

`session.pages` lists open pages and preserves host wrapper identity. Select the
page object you want to operate on; there is no process-wide active-tab state.
Popups survive opener navigation/closure and close with the session. Popup
notification follows the first navigation attempt; request failures are also
reported through the opener's error event. `current_url()` / `currentUrl()` reads
the live document URL. HTTP snapshot properties still describe requests made
through that host Page's request methods; a discovered popup initially has no host
HTTP snapshot.

The page-side popup proxy supports `closed`, `close()` and location navigation,
including reuse by name within the opener realm. Cross-window DOM access,
`opener` scripting, popup `postMessage`, window features and popup blocking policy
are not implemented. Frame execution retains the runtime's existing isolation and
layout limits.

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
  A selector action lands on the centre of the element's border box and reports
  those coordinates, so the events carry a position like a real pointer. See the
  [Python](/examples/python/#5-fill-the-form-and-save-the-edited-html) and
  [Node.js](/examples/node/#2-work-with-dom-handles) tutorials.
- Use `page.mouse` when a page reads the pointer itself: `move(x, y, steps=…)`,
  `down()`, `up()`, `click(x, y, delay=…)` and `wheel(delta_x, delta_y)`. The point
  is hit-tested against the layout, so events target the element that is there,
  with `clientX`/`clientY`/`screenX`/`screenY`, `buttons` and `isTrusted: true`.
  `steps` interpolates the travel, and `delay` holds the button down between press
  and release. Coordinates are viewport pixels.

  ```python
  box = page.evaluate("(() => { const r = document.querySelector('#go').getBoundingClientRect();"
                      " return [r.x + r.width / 2, r.y + r.height / 2]; })()")
  page.mouse.move(box[0] - 120, box[1] - 40, steps=8)
  page.mouse.click(box[0], box[1], delay=0.15)
  ```

  `document.elementFromPoint` and `document.elementsFromPoint` use the same hit
  test, so a script can look up the element at a point before acting on it.
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
cache reads and writes for matching requests. Other cache entries are separated
by proxy and credentials. See [proxies and request filtering](/guides/proxies/). The in-memory GET cache honors
max-age, no-store and no-cache; it excludes Vary and Set-Cookie responses.
Retained response bodies have separate limits: 32 MiB total and 8 MiB per body
by default.

Native streaming buffering is 256 KiB plus transport buffers. Node.js adds its
Readable buffer. Navigation and page/session closure cancel pending streams.
Buffered download delays do not throttle bytes on the wire.

</details>
