---
title: API overview
description: CLI and CDP interfaces for Brimp.
---

| Interface | Model | Best for |
| --- | --- | --- |
| [CLI](/api/cli/) | Commands launch child workers | Extraction, crawling, evaluation, screenshots, and diagnostics |
| [CDP](/api/cdp/) | Controller-managed workers | Supported Playwright and raw-CDP automation |

The lite worker delegates navigation, JavaScript, and rendering to `web-runtime`.
The CLI and controller communicate with workers across process boundaries.
See [JavaScriptCore integration](/architecture/javascript-runtime/) for the
internal Rust API and [Subsystem implementation](/architecture/subsystems/)
for engine behavior.
