---
title: Language bindings
description: Public Python and Node.js APIs, lifecycle, options, and result mapping.
---

The Python and Node.js packages are in-process adapters over the same
`web-runtime` automation API. Neither starts a CDP server or implements a second
browser. Sessions own a browser context and shared cookie jar; each page has an
owner thread and immutable document network scope. Both translate typed
commands and errors into the conventions of the host language.

## API mapping

| Operation | Python | Node.js |
| --- | --- | --- |
| Create session | `brimp.Session(...)` | `await createSession(...)` |
| Create page | `session.new_page(proxy=...)` | `await session.newPage({ proxy })` |
| Navigate | `page.get(url, ...)` | `await page.get(url, ...)` |
| Evaluate JavaScript | `page.evaluate(source)` | `await page.evaluate(source)` |
| Extract live DOM | `page.extract(...)` | `await page.extract(...)` |
| Click | `page.click(selector)` | `await page.click(selector)` |
| Hover | `page.hover(selector)` | `await page.hover(selector)` |
| Type | `page.type(selector, text)` | `await page.type(selector, text)` |
| Tap | `page.tap(selector)` | `await page.tap(selector)` |
| Read response | `response.content`, `text`, `html` | `response.content`, `text`, `html` |
| Capture PNG | `page.screenshot(...)` | `await page.screenshot(...)` |
| Close | page/session `close()` or context manager | `await page.close()` / `await session.close()` |

Both bindings use a session/page/response model. Python is synchronous; Node returns
Promises and supports `AbortSignal` cancellation for navigation. Use the
dedicated [Python reference](/api/python/) and [Node.js reference](/api/node/)
for complete signatures.

## Page options

Heavy browser subsystems are page-scoped and absent by default.

| Surface | Python `Session` | Node `createSession()` |
| --- | --- | --- |
| Workers and worklets | `enable_worker=True` | `enableWorker: true` |
| Streaming networking | `enable_streaming_networking=True` | `enableStreamingNetworking: true` |
| Persistent storage | `storage_path=...` | `storagePath: ...` |
| Storage quota | `storage_quota_bytes=...` | `storageQuotaBytes: ...` |
| Canvas 2D | `enable_canvas=True` | `enableCanvas: true` |
| WebGL | `enable_webgl=True` | `enableWebGL: true` |
| WebGPU | `enable_webgpu=True` | `enableWebGPU: true` |
| WebAudio | `enable_webaudio=True` | `enableWebAudio: true` |
| Hardware audio output | `enable_webaudio_output=True` | `enableWebAudioOutput: true` |

Enabling hardware audio output also enables WebAudio. Other options are
independent; for example, enabling WebGL does not implicitly expose Canvas 2D
or WebGPU.

## Evaluation values

Binding-level evaluation returns JSON-compatible values. Objects and arrays are
serialized structurally. The following values are unsupported across this
boundary:

- top-level `undefined`;
- functions and symbols;
- `BigInt`;
- cyclic objects; and
- objects whose serialization throws.

Use the embedded Rust [`Page::eval()` path](/architecture/javascript-runtime/#evaluate-directly-on-a-page)
when code needs a JavaScriptCore value handle instead of JSON serialization.

## Resource lifetime

Close bindings explicitly. Python sessions and pages support `with`; Node
sessions and pages expose idempotent asynchronous `close()` methods. Closing a
session closes all its pages and joins their owner threads. Operations after close fail with the binding's
stable `closed` error category.

## Choosing a binding or CDP

- Choose Python for synchronous request/response extraction.
- Choose Node for asynchronous request/response extraction and cancellation.
- Choose CDP when an existing Playwright or Puppeteer workflow only needs
  Brimp's documented protocol subset.
- Choose Rust when embedding the page/runtime directly or supplying a custom
  `ResourceLoader`.
