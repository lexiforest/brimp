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
| Create browser/session | `brimp.Session(...)` | `await launch(...)` |
| Create page | Session owns one page | `await browser.newPage(...)` |
| Navigate | `session.get(url, ...)` | `await page.goto(url, ...)` |
| Evaluate JavaScript | `session.evaluate(source)` | `await page.evaluate(source)` |
| Read title | Evaluate `document.title` | `await page.title()` |
| Read document text | Evaluate `document.documentElement.textContent` | `await page.textContent()` |
| Capture PNG | `session.screenshot(...)` | `await page.screenshot(...)` |
| Close | `session.close()` or context manager | `await page.close(); await browser.close()` |

Python follows a synchronous Requests-style session/response model. Node uses
an asynchronous browser/page model and supports `AbortSignal` cancellation for
navigation. Use the dedicated [Python reference](/api/python/) and [Node.js
reference](/api/node/) for complete signatures.

## Page options

Heavy browser subsystems are page-scoped and absent by default.

| Surface | Python `Session` | Node `browser.newPage()` |
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

Close bindings explicitly. Python sessions support `with`; Node pages and
browsers expose idempotent asynchronous `close()` methods. Closing the browser
closes its child pages and joins their owner threads. Operations after close
fail with the binding's stable `closed` error category.

## Choosing a binding or CDP

- Choose Python for synchronous request/response extraction and persistent
  cookies/connections.
- Choose Node for asynchronous in-process navigation and cancellation.
- Choose CDP when an existing Playwright or Puppeteer workflow only needs
  Brimp's documented protocol subset.
- Choose Rust when embedding the page/runtime directly or supplying a custom
  `ResourceLoader`.
