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
| [CDP](/api/cdp/) | HTTP/WebSocket server | The supported Puppeteer connection workflow |

## Shared behavior

- Navigation loads the main response, scripts, stylesheets, and common image resources.
- JavaScript evaluation returns JSON-compatible values.
- Screenshots are PNG and can cover the viewport or full page where exposed.
- Browser/page/session objects own native resources and should be closed explicitly.
- Personas apply a coherent transport, header, navigator, screen, and viewport identity.

The interfaces intentionally differ in shape. Python follows a request/response
model, Node follows browser/page automation, the CLI is process-oriented, and
CDP serializes values across a remote boundary.
