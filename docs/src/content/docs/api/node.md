---
title: Node API
description: Find Node.js methods, options, events and TypeScript types.
---

[Open the generated Node API reference →](/reference/node/)

Browse session options, request options, page methods, event records, routes and
plugins. The reference includes TypeScript types for each interface.

For runnable ESM examples, follow the [Node tutorial](/examples/node/).

Request methods return `Page`, which provides HTTP metadata, the original
response body, and live DOM/JavaScript operations. Call `page.raiseForStatus()`
to throw `HTTPError` for a 4xx/5xx status. See
[sessions, pages and requests](/guides/automation/) for response lifetimes.

Evaluation supports bigint, undefined, special numbers and binary buffers.
JSHandle supports nested reference arguments, property mutation and invocation
with an explicit receiver. See the
[value bridge guide](/guides/values/)
for plugin functions, ownership and WebAssembly loading.

## Browser profiles

`page.profileContract()` and `page.profileReport()` describe the loaded interface definitions and report
defaulted values and capture provenance. See [browser capture profiles](/guides/profiles/).

## Saved HTML

`page.loadHtml(html, url = 'about:blank')` installs `url` before parsing and
runs no inline or external page scripts and fetches no subresources. Init
scripts still run. Use it to call an extracted script against a saved document
with correct relative URL and document URL resolution.
