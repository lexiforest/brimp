---
title: Python API
description: Reference for the synchronous brimp Python package.
---

```python
import brimp
```

The Python binding is synchronous and in process. Pages in one session can run
concurrently from separate threads while sharing the session cookie jar.

## `brimp.get()`

```python
brimp.get(url, **options) -> Response
```

Creates a temporary [`Session`](#session), performs one GET navigation, closes
the native session, and returns a detached response. It accepts the same options
as `Page.get()` plus `proxy`, `persona_json`, and `ca_bundle` for page/session creation.

## `Session`

```python
brimp.Session(
    *,
    persona_json: str | None = None,
    ca_bundle=None,
    enable_worker: bool = False,
    enable_streaming_networking: bool = False,
    enable_canvas: bool = False,
    enable_webgl: bool = False,
    enable_webgpu: bool = False,
    enable_webaudio: bool = False,
    enable_webaudio_output: bool = False,
    storage_path=None,
    storage_quota_bytes: int | None = None,
)
```

Creates a persistent browsing session. `persona_json` is JSON text following
Brimp's versioned persona schema. `ca_bundle` is a PEM file used to trust
private or enterprise certificate authorities without disabling certificate or
hostname verification.

Every browser subsystem is disabled by default. `enable_worker` enables the
worker-family APIs. `enable_streaming_networking` enables WebSocket, EventSource,
streaming Fetch, and the stream classes. Canvas 2D, WebGL, WebGPU, and WebAudio
are enabled independently by `enable_canvas`, `enable_webgl`, `enable_webgpu`,
and `enable_webaudio`. Supplying `storage_path` enables origin-partitioned
persistent storage at that directory; its default quota is 1 GiB and
`storage_quota_bytes` overrides it.
`enable_webaudio` retains the device-free sink. Setting
`enable_webaudio_output=True` also enables WebAudio and authorizes playback
through the system output device.

Sessions expose mutable request seed `headers` and `cookies` dictionaries and
support the context-manager protocol.

### `Session.new_page()`

```python
session.new_page(proxy: str | None = None) -> Page
```

Creates an independently concurrent page. Its direct, HTTP, SOCKS5, or SOCKS5H
proxy is immutable and applies to the main request, redirects, subresources,
Fetch, workers, streaming requests, and WebSockets for the page's lifetime.

### `Page.get()`

```python
page.get(
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

Session and call cookies are inserted into the browser-managed cookie jar before
navigation, so normal domain, path, redirect, and expiry rules apply. Response
cookies persist in the native shared jar and are exposed on the response.

### `Page.evaluate()`

```python
page.evaluate(expression: str) -> object
```

Evaluates JavaScript in the current page and returns a JSON-compatible Python
value. JavaScript exceptions raise `JavaScriptError`; unsupported result values
raise a `BrimpError` with the corresponding native code.

### `Page.screenshot()`

```python
page.screenshot(path=None, *, full_page: bool = False) -> bytes
```

Returns PNG bytes. When `path` is supplied, it writes the same bytes to that
path.

### `Page.extract()`

```python
page.extract(
    *,
    content_selector: str | None = None,
    remove_images: bool = False,
    language: str | None = None,
    debug: bool = False,
) -> dict
```

Runs the pinned Defuddle browser bundle against the current live DOM and
returns extracted content, Markdown, and metadata. It does not reparse the page
with jsdom or make another network request.

### Page input

```python
page.click(selector: str) -> None
page.hover(selector: str) -> None
page.type(selector: str, text: str) -> None
page.tap(selector: str) -> None
```

These methods hit-test and send trusted browser input events. `hover()` moves
without pressing, while `type()` focuses
the matched control before sending keyboard/editing input. A missing selector
raises `InvalidRequest` with code `invalid_input`.

### `Session.close()`

Closes every page and native resource. `Page.close()` closes only that page.
Closing more than once is safe. Operations after close
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
