---
title: API overview
description: Public interfaces and support boundaries for Brimp.
---

Brimp exposes four automation interfaces backed by the same runtime:

| Interface | Model | Best for |
| --- | --- | --- |
| [Python](/api/python/) | Synchronous, in-process | Requests-style extraction and scripts |
| [Node.js](/api/node/) | Asynchronous, in-process | Node applications and cancellable navigation |
| [CLI](/api/cli/) | One command per operation | Shell automation and diagnostics |
| [CDP](/api/cdp/) | HTTP/WebSocket server | Supported Playwright, Puppeteer, and raw-CDP workflows |
| [Rust page API](/architecture/javascript-runtime/#evaluate-directly-on-a-page) | Embedded, owner-thread-bound | Direct JSC values and custom resource loaders |

## Shared behavior

- Navigation loads the main response, scripts, stylesheets, and common image resources.
- JavaScript evaluation returns JSON-compatible values.
- Screenshots are PNG and can cover the viewport or full page where exposed.
- Session and browser/page objects own native resources and should be closed explicitly.
- Personas apply a coherent transport, header, navigator, screen, and viewport identity.

The native bindings share a request/response model: Python is synchronous and
Node is asynchronous with `AbortSignal` cancellation. The CLI is
process-oriented, while CDP provides browser/page automation across a remote
boundary.

See [Language bindings](/api/bindings/) for a side-by-side lifecycle and option
mapping, [JavaScriptCore integration](/architecture/javascript-runtime/) for
evaluation internals, and [Subsystem implementation](/architecture/subsystems/)
for backend and feature-gate details.
