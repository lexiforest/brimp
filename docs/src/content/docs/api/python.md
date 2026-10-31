---
title: Python API
description: Find Python methods, response properties, handles and plugin APIs.
---

[Open the generated Python API reference →](/reference/python/)

Use it to look up request methods, response properties, handles, exceptions and
plugin configuration.

For a runnable introduction, follow the [Python tutorial](/examples/python/).

Request methods return `brimp.Page`, which provides HTTP metadata, the original
response body, and live DOM/JavaScript operations. Call `page.raise_for_status()`
to raise `brimp.HTTPError` for a 4xx/5xx status. See
[sessions, pages and requests](/guides/automation/) for response lifetimes.

Evaluation copies undefined/null, BigInt, special numbers and binary buffers.
Use JSHandle for identity, functions and cyclic values. The
[value bridge guide](/guides/values/)
covers reference calls, plugin callbacks, ownership and WebAssembly loading.

## Browser profiles

`page.profile_contract()` and `page.profile_report()` describe the loaded interface definitions and report
defaulted values and capture provenance. See [browser capture profiles](/guides/profiles/).

## Saved HTML

`page.load_html(html, url="about:blank")` installs `url` before parsing and
runs no inline or external page scripts and fetches no subresources. Init
scripts still run. Use it to call an extracted script against a saved document
with correct relative URL and document URL resolution.
