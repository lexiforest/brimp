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

## CLI and CDP

The CLI launches a worker for browser operations. `brimp serve` exposes a CDP
endpoint and manages workers for connected clients. The lite worker uses
`runtime`; a separately built WebKit worker is another engine option.
Neither the CLI nor controller initializes the lite engine in its own process.

## When to use Brimp

Brimp is a good fit when you need JavaScript-rendered HTML, structured
evaluation, screenshots, persistent cookies, or a supported Playwright
workflow without launching Chromium.

Brimp is not a complete replacement for every browser automation workload. It
implements a deliberate subset of web APIs and CDP. It does not expose a visual
browser UI, multiple browser processes, arbitrary Chrome extensions, or the
complete DevTools Protocol. Unsupported CDP methods return protocol error
`-32601` instead of pretending to succeed.

See the [API overview](/api/) for the exact public interfaces and their current
limits.
