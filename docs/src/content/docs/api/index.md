---
title: API overview
---

The native packages expose requests-style Session/Page APIs. Sessions isolate
cookies and plugin descriptors; pages own their realms and DOMs. Python uses
snake_case, bytes, context managers, and typed exceptions. Node uses camelCase,
Promises, Buffer/Uint8Array, TypeScript declarations, and request/stream AbortSignal.

| Capability | Python / Node |
| --- | --- |
| Navigation | `session.get/post/...`, reusable `page.request` |
| Evaluation | `evaluate`, `evaluate_handle` / `evaluateHandle` |
| DOM handles | `query_selector` / `querySelector`, plural forms, attributes, text/HTML, edits/removal |
| Input | `click`, `hover`, `tap`, `focus`, `fill`, `press`, `type_text` / `typeText`, `drag_to` / `dragTo`, file inputs |
| Events | `on` returns unsubscribe; console/error, lifecycle, mutation, network, WebSocket/EventSource |
| Routes | `route` returns removal; request/response/auth stages, exactly one decision |
| Streams | `page.stream`: Python chunk iterator / Node Readable |
| Controls | headers, identity, init scripts, cache/offline/latency, bounded retained bodies |
| Extraction | `extract` requires the explicit Defuddle plugin |
| Plugin discovery | `plugins.catalog()` / `pluginCatalog()` reports capabilities, options and lifetimes |
| Host storage | `plugins.host_storage(...)` / `hostStoragePlugin(...)` installs async origin-partitioned storage with a configurable backend |

Evaluation pumps timers, network and host completions while awaiting Promises.
Callbacks cannot reenter the same page. Python callbacks run synchronously on a
managed dispatcher; Node callbacks may return Promises. Routes abort if a handler
throws or returns without deciding. Response interception buffers at most 32 MiB.
Native stream consumers buffer at most 256 KiB plus transport buffers. Navigation,
closure, and cancellation terminate pending operations.

The page realm includes DOM lifecycle and Range operations, FileReader, Web
Streams, structured cloning, XPath, Shadow DOM and WebCrypto. See
[core and plugin coverage](/architecture/subsystems/) for supported behavior.

Layout, screenshots, coordinate actions, XML, browser Workers, and native
graphics/audio engines are unsupported. Durable storage requires an explicit
hostStorage backend; the browser storage plugin is realm-local memory.
Full source examples and declarations are in `bindings/`.
