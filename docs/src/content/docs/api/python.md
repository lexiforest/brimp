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

For one-off requests, use `brimp.get(url, params={"q": "browser"})` in a `with`
block. Closing its Page also closes the helper's private session. For shared
settings, use `brimp.Session(base_url="https://example.com/", params={"lang": "en"})`
and request relative URLs. Navigation and streaming accept per-request `params`
and `quote` controls.

For asyncio, use `brimp.AsyncSession` and await requests, page operations and
handle methods. It returns `brimp.AsyncPage`; HTTP metadata, `json()` and
`raise_for_status()` remain synchronous. See the
[async request example](/guides/automation/#use-python-with-asyncio).

Evaluation copies undefined/null, BigInt, special numbers and binary buffers.
Use JSHandle for identity, functions and cyclic values. The
[value bridge guide](/guides/values/)
covers reference calls, plugin callbacks, ownership and WebAssembly loading.

Use session or per-request `proxy`, `proxies`, `proxy_auth` and `trust_env`
options to choose a proxy. `session.route()` can filter requests or override the
proxy for each resource. See [proxies and request filtering](/guides/proxies/).

Catch `brimp.BrimpError` for Brimp failures or `brimp.RequestException` for HTTP
failures. Typed network exceptions retain numeric curl codes; HTTP status and
JSON decoding exceptions attach their Page at `.response` and `.page`.
See the [exception reference](/reference/python/errors/).

## Request bodies

`session.post()` and other request methods accept `json=` for JSON, `content=`
for raw strings/bytes, and `data=` for raw strings/bytes or URL-encoded form
mappings/pairs. These arguments also work with module helpers, async sessions,
and `page.stream()`. Explicit `Content-Type` headers override encoding defaults.
See [body examples](/examples/python/#3-send-forms-json-or-raw-bytes).

Bodies are buffered. File-like, iterable and multipart uploads remain unsupported.
GET/HEAD navigation still rejects request bodies.

## Timeouts

`timeout` accepts nonnegative numeric seconds (including fractions), or `None`/`0` to disable the deadline.
For example, `session.get(url, timeout=10)` gives the operation ten seconds. Omitting the request argument inherits the session default (30 seconds).
An explicit `timeout=None` overrides a finite session default.

Brimp uses one deadline for navigation, including JavaScript. Timeout tuples
and separate connection/read timers are unsupported.
`page.stream()` uses the same budget through response consumption. The formats
also work with module helpers and `brimp.AsyncSession`; cancellation and closing
still work when the deadline is disabled.

## Browser profiles

Pass an editable `brimp.Fingerprint` as `profile=`, or combine it with browser
values at `profile["values"]["network"]["fingerprint"]`. Pro catalog updates and
reads belong to the CLI. See [Pro fingerprints](/guides/profiles/#pro-fingerprints).

`page.profile_contract()` and `page.profile_report()` describe the loaded interface definitions and report
defaulted values and capture provenance. See [browser capture profiles](/guides/profiles/).

## Saved HTML

`page.load_html(html, url="about:blank")` installs `url` before parsing and
runs no inline or external page scripts and fetches no subresources. Init
scripts still run. Use it to call an extracted script against a saved document
with correct relative URL and document URL resolution.
