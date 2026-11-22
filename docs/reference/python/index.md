# Python API

Generated from the Python source; no native library is imported during this build.
Use [the Python tutorial](/examples/python/) for runnable examples. Timeouts use seconds.

## HTTP helpers

Helpers return a live Page with a private session. Close the page or use a context
manager to release both. Use Session for shared cookies and session configuration.

::: brimp.request

::: brimp.get

::: brimp.head

::: brimp.post

::: brimp.put

::: brimp.patch

::: brimp.delete

::: brimp.options

::: brimp.trace

::: brimp.query

## Sessions and pages

::: brimp.Session

::: brimp.Page

::: brimp.JSHandle

::: brimp.ElementHandle

::: brimp.Mouse

::: brimp.Route

::: brimp.StreamingResponse

## Locators and frames

Locators resolve the current DOM on each operation. Frame scopes resolve a selector
path; their methods return awaitables when created from an AsyncPage. See the
[automation guide](/guides/automation/) for event expectations and popup ownership.

::: brimp.Locator

::: brimp.Frame

## Async sessions and pages

Use `async with brimp.AsyncSession(...)` and await requests, page operations and
handle methods. Buffered HTTP properties, `json()` and `raise_for_status()` stay
synchronous. Page callback registration stays synchronous; session routes use
`await session.route(...)`. Handlers and removal functions remain synchronous. Async streaming and coroutine callbacks are not yet supported.

::: brimp.AsyncSession

::: brimp.AsyncPage

::: brimp.AsyncJSHandle

::: brimp.AsyncElementHandle

## Fingerprint data

The CLI manages fingerprint catalogs. This editable data class can be supplied as
`Session(profile=...)` or embedded in a browser profile's `network.fingerprint`.

::: brimp.Fingerprint
