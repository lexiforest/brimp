---
title: Subsystem implementation
description: Backends, ownership, data flow, feature gates, and deliberate boundaries.
---

Brimp has one page implementation in `brimp-runtime`. Lite CDP dispatch calls that implementation; the CLI communicates with workers. Browser subsystems share the
page's canonical DOM, task queue, resource loader, persona, and owner-thread
rules rather than maintaining parallel state.

## Implementation map

| Subsystem | Implementation | Page gate |
| --- | --- | --- |
| JavaScript | WebKit JavaScriptCore through `brimp-jsc` | Always present |
| DOM, HTML parsing, CSS and layout | Blitz DOM/html, Stylo, and `brimp-dom` | Always present |
| Navigation and resources | `brimp-runtime` policy over `brimp_network::ResourceLoader` | Always present |
| HTTP, TLS, HTTP/2 and HTTP/3 | libcurl-impersonate multi executor | Always present |
| Events, timers and basic Fetch | JavaScript bindings plus the page task queue | Always present |
| Workers and worklets | Isolated JSC runtimes and a browser-owned coordinator | `worker_system` |
| WebSocket, EventSource and streaming Fetch | curl-impersonate streaming handles with bounded backpressure | `streaming_networking` |
| Persistent web storage | Origin partitions, quota enforcement, and filesystem backing | `persistent_storage` |
| Canvas 2D | Raster-only `skia-safe`; Rustybuzz and bundled fonts for text | `canvas` |
| Document screenshots | Blitz paint through Vello CPU, encoded as PNG | Always present |
| Persona | One resolved identity shared by transport and Web APIs | Always present |

All optional gates default to off and are page-scoped. The CLI,
CDP worker configuration, and Rust `PageOptions` expose the same independent
choices.

## Page lifecycle and ownership

`brimp-runtime::Page` owns the JavaScriptCore context, `BrowserDocument`, viewport,
bindings, task queues, loader state, and optional backend stores. Navigation
creates a fresh document and JavaScript realm, applies the resolved persona,
loads the main response, pauses parsing for parser-blocking classic scripts,
runs deferred scripts, and settles the page lifecycle after discovered
resources finish.

`AutomationPage` keeps that state on a dedicated owner thread. External APIs
send commands to it; they do not duplicate navigation or rendering behavior.

## DOM, CSSOM, and layout

`brimp-dom` owns one canonical Blitz tree. JavaScript `Node`, `Element`, and
`Document` objects contain stable mappings to Blitz node IDs; there is no mirror
DOM. Mutations from JavaScript therefore immediately affect parsing, selectors,
style resolution, layout, and screenshots.

Incremental HTML parsing uses Blitz/html5ever and yields at parser-inserted
scripts. CSS parsing and computed style use Stylo. Geometry reads resolve style
and layout before returning values. CSSOM wrappers retain stylesheet/rule
identity while mutations update the underlying author stylesheets.

## Networking and navigation

Every main document, stylesheet, classic script, image, font, and JavaScript
Fetch request crosses `brimp_network::ResourceLoader`. The default
`CurlResourceLoader` runs libcurl-impersonate easy handles on one bounded
curl-multi executor and pools handles for reuse.

libcurl owns transport mechanics and browser-profile TLS/HTTP behavior.
`brimp-runtime` owns browser policy: URL resolution, context-owned cookie jars shared
by pages, redirect hops,
persona headers, response limits, cancellation, and lifecycle events. This
separation also lets tests or embedders inject deterministic loaders.

The streaming gate adds incremental body delivery, WebSocket, and EventSource.
Queues are bounded; consumers can pause/resume curl transfers and cancellation
removes the active transfer.

## Workers

Dedicated workers run isolated JavaScriptCore runtimes. Shared-worker and
service-worker registrations live in a browser-owned coordinator so named
realms can outlive a page. Messages and lifecycle events cross queues and are
delivered on the receiving realm's owner thread. The current worker layer is a
selected functional surface, not complete Worker/Web Worker WPT conformance.

## Persistent storage

Supplying a storage root enables origin-partitioned IndexedDB, Cache Storage,
StorageManager/quota, OPFS, and durable `localStorage`. The storage layer uses
the page origin as its partition key and enforces the configured quota. Without
a root, the persistent APIs are absent; ordinary page-lifetime storage remains
separate.

## Canvas 2D

Canvas uses raster-only Skia surfaces through `skia-safe`. Skia owns pixels,
paths, clipping, compositing, gradients, patterns, shadows, filters, image
operations, and encoding. Canvas does not initialize Skia's GPU backends.

Text is shaped with Rustybuzz, Unicode bidi and grapheme handling, using the
bundled WQY Micro Hei proportional/monospace faces and Noto Color Emoji for text.
Canvas maintains origin-clean state for image sources and supplies consistent
pixels to `getImageData()`, export methods, and document screenshots. Completed
Canvas rasters enter the document's Vello compositor as images.

## Screenshots

Document screenshots resolve the Blitz layout, paint through `blitz-paint` and
AnyRender's Vello CPU backend, composite Canvas rasters, and encode
the final RGBA image as PNG. No native window or display server is required.
Viewport screenshots use the configured viewport; full-page screenshots
temporarily extend rendering to the document content bounds.

## Persona coherence

A persona is resolved once when the browsing context is created. Its transport
profile, ordered request headers, Navigator values, language, screen, viewport,
and Canvas behavior feed the relevant subsystems from one snapshot.

For dependency-backed implementation status, see
[`SUBSYSTEMS.md`](https://github.com/lexiforest/brimp/blob/main/SUBSYSTEMS.md).
That ledger distinguishes completed selected surfaces from full
browser-standard conformance and records deliberate exclusions such as video.
Core DOM and JavaScriptCore compatibility checks remain in
[`PATCH_ENV.md`](https://github.com/lexiforest/brimp/blob/main/PATCH_ENV.md).
