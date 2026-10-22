---
title: Introduction
---

**A JavaScript runtime for JavaScript from the browser.**

Brimp combines JavaScriptCore,
Lexbor, and curl-impersonate behind synchronous Python and asynchronous Node APIs.
It runs page scripts and maintains a live DOM without rendering or layout.

Common web APIs belong in core: DOM parsing and mutation, events, script loading,
Fetch/XHR, Web Streams, FileReader, structured cloning, and WebCrypto. Pages run
in independent JavaScript realms, with handles and typed events exposed to the host.

Plugins add capabilities for specific workloads. Install bundled plugins or
provide your own JavaScript installer and host handlers. Storage lifetimes and
mock behavior are explicit; no plugins are enabled by default.

Start with the [native quick start](/quick-start/), [API](/api/), and
[plugin support](/architecture/subsystems/).
