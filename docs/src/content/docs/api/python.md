---
title: Python API
description: Reference for the synchronous brimp Python package.
---

```python
import brimp
```

The Python binding is synchronous and in process. Sessions are sequential and
are not thread-safe.

## `brimp.get()`

```python
brimp.get(url, **options) -> Response
```

Creates a temporary [`Session`](#session), performs one GET navigation, closes
the native session, and returns a detached response. It accepts the same options
as `Session.get()` plus `persona_json` and `ca_bundle` for session creation.

## `Session`

```python
brimp.Session(
    *,
    persona_json: str | None = None,
    ca_bundle=None,
    enable_worker: bool = False,
    enable_streaming_networking: bool = False,
    storage_path=None,
    storage_quota_bytes: int | None = None,
)
```

Creates a persistent browsing session. `persona_json` is JSON text following
Brimp's versioned persona schema. `ca_bundle` is a PEM file used to trust
private or enterprise certificate authorities without disabling certificate or
hostname verification.

The three browser subsystems are disabled by default. `enable_worker` enables
the worker-family APIs. `enable_streaming_networking` enables WebSocket,
EventSource, streaming Fetch, and the stream classes. Supplying `storage_path`
enables origin-partitioned persistent storage at that directory; its default
quota is 1 GiB and `storage_quota_bytes` overrides it.

Sessions expose mutable `headers` and `cookies` dictionaries and support the
context-manager protocol.

### `Session.get()`

```python
session.get(
    url,
    *,
    params=None,
    headers=None,
    cookies=None,
    timeout: float = 30.0,
) -> Response
```

Performs a GET navigation with a fresh JavaScript realm. Query parameters use
standard URL encoding and support repeated values. Session headers are merged
with call headers; call values take precedence. `User-Agent` and
`Accept-Language` are persona-owned and cannot be overridden here.

Session cookies, call cookies, and browser-managed cookies are sent together.
Response cookies update `session.cookies`.

### `Session.evaluate()`

```python
session.evaluate(expression: str) -> object
```

Evaluates JavaScript in the current page and returns a JSON-compatible Python
value. JavaScript exceptions raise `JavaScriptError`; unsupported result values
raise a `BrimpError` with the corresponding native code.

### `Session.screenshot()`

```python
session.screenshot(path=None, *, full_page: bool = False) -> bytes
```

Returns PNG bytes. When `path` is supplied, it writes the same bytes to that
path.

### `Session.close()`

Closes native resources. Closing more than once is safe. Operations after close
raise a `BrimpError` with code `closed`.

## `Response`

| Member | Meaning |
| --- | --- |
| `status_code` | Final HTTP status code. |
| `reason` | HTTP reason phrase. |
| `url` | Final response URL after redirects. |
| `headers` | Case-insensitive [`Headers`](#headers) mapping. |
| `content` | Original final response bytes. |
| `text` | Original response decoded from its declared charset, or UTF-8. |
| `html` | Post-JavaScript serialized DOM for HTML responses; otherwise `None`. |
| `cookies` | Cookies received with the response. |
| `elapsed` | Native request/navigation elapsed value. |
| `ok` | `True` when the status is below 400. |

`response.json()` decodes `response.text`. `response.raise_for_status()` raises
`HTTPError` for 4xx and 5xx responses; navigation itself does not raise solely
because of an HTTP error status.

## `Headers`

`Headers` implements `collections.abc.Mapping`. Lookup is case-insensitive,
duplicate values are comma-joined, `get_all(name)` returns every value, and
`raw` returns the original `(name, value)` entries.

## Exceptions

All Brimp exceptions derive from `BrimpError`, which derives from `OSError`.

| Exception | Typical condition |
| --- | --- |
| `ConnectionError` | Transport failure. |
| `Timeout` | Navigation exceeded its timeout. |
| `TooManyRedirects` | Redirect limit reached. |
| `InvalidRequest` | Invalid arguments or request configuration. |
| `InvalidURL` | URL validation failure. |
| `HTTPError` | Explicit `raise_for_status()` on a 4xx/5xx response. |
| `JavaScriptError` | Explicit evaluation threw an exception. |

## Current boundary

The initial Python API supports GET only. It does not expose POST bodies,
streaming, multipart uploads, prepared requests, transport adapters,
per-request proxies, concurrent session use, or an asynchronous facade.
