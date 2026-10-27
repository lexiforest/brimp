---
title: Browser support
description: Supported browser APIs, partial implementations, and known limits.
---

Brimp runs browser JavaScript with a DOM and network stack. It can execute scripts,
update a document, submit supported requests, and return the resulting content.
It does not render the page: there is no layout, CSS cascade, paint engine or
browser user interface.

The core APIs below are available through Python, Node and the CLI. Browser subsystem
[plugins](/guides/plugins/) are enabled by default; extraction and host storage
are selected explicitly. An explicit plugin list replaces the defaults.

## Find an API family

| Area | What works | Main limits |
| --- | --- | --- |
| [DOM and documents](/web-platform/dom/) | HTML parsing, selectors, mutation, live collections, ranges, XPath, Shadow DOM and autonomous custom elements | No XML parser; partial Shadow DOM and custom-element behavior |
| [Forms and input](/web-platform/forms/) | Values, checkedness, form association, validation, selection, numeric/date stepping and direct input | GET form submission; no pickers, validation UI or layout-based input |
| [Events](/web-platform/events/) | Dispatch, propagation, cancellation, handlers, observers and event payloads | Many device/UI events can only be dispatched manually |
| [Window and environment](/web-platform/environment/) | Navigation, history state, timers, profile identity, media queries and font metadata | Profile values are configured; no browser UI, media playback or worker engine |
| [CSS, SVG and geometry](/web-platform/css-svg/) | Inline declarations, editable stylesheets, SVG DOM and attribute reflection | No computed style, layout, SVG renderer or animation engine |
| [Network, files and streams](/web-platform/network-files/) | Fetch/XHR, Blob/File/FileReader, streams, structured cloning and message channels | Partial browser policies and body formats; cloning can buffer data |
| [Cryptography](/web-platform/crypto/) | Random values, UUIDs and the documented WebCrypto algorithms | No Ed25519/X25519 or CryptoKey cloning |

For JavaScript values crossing into Python or Node, see [Values and handles](/guides/values/).
That host bridge has a different contract from page-side `structuredClone`.

## How to read support claims

- **Implemented:** the operation performs the behavior described on its topic page.
  This does not imply the entire web standard is implemented.
- **Profile-based:** the result comes from the configured browser profile rather
  than a live screen, device or operating system measurement.
- **Placeholder:** the member exists but returns a fixed value or performs no work.
  A callable method may return `undefined` even where a browser returns a Promise.
- **Unavailable:** the interface or capability is absent or rejects the operation.

Check behavior as well as presence. For example, `element.scrollIntoView()` is a
no-op, `getComputedStyle()` returns `undefined`, and media `play()` rejects.
These methods cannot be used as evidence of rendering or playback support.

## Examples and detailed contracts

Each topic starts with practical capabilities and limits. Expand **Detailed
behavior and edge cases** when you need conversion rules, state lifetime or a
specific difference from Safari. The [runtime test suite](https://github.com/lexiforest/brimp/tree/main/tests/runtime/)
and [field ledger](https://github.com/lexiforest/brimp/blob/main/COVERAGE.yaml)
provide the corresponding evidence.

Browser-library regressions exercise real applications and packages. Detector
results remain observations; they do not define new product capabilities or
promise that Brimp is indistinguishable from a browser.
