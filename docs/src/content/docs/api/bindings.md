---
title: Language bindings
description: Public Python and Node.js APIs, lifecycle, options, and result mapping.
---

The Python and Node.js packages are in-process adapters over the same
`web-runtime` automation API. Neither starts a CDP server or implements a second
browser. Both create a native page on its owner thread and translate typed
commands and errors into the conventions of the host language.

## API mapping

| Operation | Python | Node.js |
| --- | --- | --- |
| Create session | `brimp.Session(...)` | `await createSession(...)` |
| Native page | Session owns one page | Session owns one page |
| Navigate | `session.get(url, ...)` | `await session.get(url, ...)` |
| Evaluate JavaScript | `session.evaluate(source)` | `await session.evaluate(source)` |
| Click | `session.click(selector)` | `await session.click(selector)` |
| Hover | `session.hover(selector)` | `await session.hover(selector)` |
| Type | `session.type(selector, text)` | `await session.type(selector, text)` |
| Tap | `session.tap(selector)` | `await session.tap(selector)` |
| Read response | `response.content`, `text`, `html` | `response.content`, `text`, `html` |
| Capture PNG | `session.screenshot(...)` | `await session.screenshot(...)` |
| Close | `session.close()` or context manager | `await session.close()` |

Both bindings use a session/response model. Python is synchronous; Node returns
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

Close bindings explicitly. Python sessions support `with`; Node sessions expose
an idempotent asynchronous `close()` method. Closing a session closes its page
and joins the owner thread. Operations after close fail with the binding's
stable `closed` error category.

## Choosing a binding or CDP

- Choose Python for synchronous request/response extraction.
- Choose Node for asynchronous request/response extraction and cancellation.
- Choose CDP when an existing Playwright or Puppeteer workflow only needs
  Brimp's documented protocol subset.
- Choose Rust when embedding the page/runtime directly or supplying a custom
  `ResourceLoader`.
