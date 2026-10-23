---
title: Introduction
---

**A JavaScript runtime for JavaScript from the browser.**

Brimp combines JavaScriptCore,
Lexbor, and curl-impersonate behind an in-process CLI, synchronous Python APIs
and asynchronous Node APIs.
It runs page scripts and maintains a live DOM without rendering or layout.

## Common web in core, specialized APIs in plugins

Common web APIs belong in core: DOM parsing and mutation, forms, non-rendering stylesheet editing, events, script loading,
Fetch/XHR, Web Streams, FileReader, structured cloning, and WebCrypto. Pages run
in independent JavaScript realms, with handles and typed events exposed to the host.

Plugins add capabilities for specific workloads. Install bundled plugins or
provide your own JavaScript installer and host handlers. Storage lifetimes and
mock behavior are explicit. Default binding sessions install no plugins; the CLI
automatically installs extraction for Markdown/JSON output.

Start with the [quick start](/quick-start/), [API](/api/), and
[plugin support](/architecture/subsystems/).

## Why Brimp

Many sites return an HTML shell and rely on JavaScript to produce the useful
content. A request library retrieves the shell but does not execute the page.
Chromium solves that problem, but its process model and resource footprint are
often excessive for extraction agents and bounded automation jobs.

Brimp keeps a request/response workflow while adding the browser layers those
pages need:

- JavaScript execution and DOM mutation;
- inline styles and non-rendering stylesheet editing;
- navigation, subresources, cookies, timers, events, and `fetch()`;
- curl-impersonate transport profiles for HTTP/TLS behavior; and
- coherent, configurable browser personas.

## One runtime, 3 interfaces

The CLI, Python binding, Node binding all delegate to the same
`brimp-runtime` automation boundary. They do not contain separate browser
implementations.

![The CLI, Python and Node interfaces share brimp-runtime, which composes brimp-jsc and JavaScriptCore, brimp-lexbor and Lexbor, and brimp-net and curl-impersonate.](/diagrams/runtime-layers.png)

All three interfaces run the runtime in process. Each page owns its JavaScript
realm and DOM; a session shares cookies, connections and plugin configuration.

## When to use Brimp

Brimp is a good fit when you need JavaScript-rendered HTML, structured
evaluation, cookies shared across requests, or a small DOM automation
workflow without launching Chromium.

Brimp is not a complete replacement for every browser automation workload. It
implements a deliberate subset of web APIs. It does not expose a visual
browser UI, layout, screenshots, browser worker processes, Chrome extensions,
or a CDP server. Choose a full browser such as Chromium through Playwright
when your task needs those capabilities.

See the [API overview](/api/) for the exact public interfaces and their current
limits.
