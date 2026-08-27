---
title: Introduction
description: What Brimp is, how it works, and when to use it.
---

Brimp is a lightweight, headless browser for agents. It fills the gap between a
plain HTTP client and a full Chromium automation stack: pages are fetched with
browser-like HTTP/TLS fingerprints, parsed into a live DOM, executed with
JavaScriptCore, laid out, and optionally rendered to PNG.

## Why Brimp

Many sites return an HTML shell and rely on JavaScript to produce the useful
content. A request library retrieves the shell but does not execute the page.
Chromium solves that problem, but its process model and resource footprint are
often excessive for extraction agents and bounded automation jobs.

Brimp keeps a request/response workflow while adding the browser layers those
pages need:

- JavaScript execution and DOM mutation;
- styles, layout queries, and CPU screenshots;
- navigation, subresources, cookies, timers, events, and `fetch()`;
- curl-impersonate transport profiles for HTTP/TLS behavior; and
- coherent, configurable browser personas.

## One runtime, four interfaces

The CLI, Python binding, Node binding, and CDP server all delegate to the same
`web-runtime` automation boundary. They do not contain separate browser
implementations.

```text
CLI          Python          Node          CDP / Puppeteer
 │              │              │                  │
 └──────────────┴──────────────┴──────────────────┘
                         │
                  Brimp web runtime
                         │
      JavaScriptCore + Blitz DOM/layout + screenshots
                         │
                  curl-impersonate
```

Python and Node run in process. CDP is the remote JSON/WebSocket boundary.

## When to use Brimp

Brimp is a good fit when you need JavaScript-rendered HTML, structured
evaluation, screenshots, persistent cookies, or a small Puppeteer-compatible
workflow without launching Chromium.

Brimp is not a complete replacement for every browser automation workload. It
implements a deliberate subset of web APIs and CDP. It does not expose a visual
browser UI, multiple browser processes, arbitrary Chrome extensions, or the
complete DevTools Protocol. Unsupported CDP methods return protocol error
`-32601` instead of pretending to succeed.

See the [API overview](/api/) for the exact public interfaces and their current
limits.
