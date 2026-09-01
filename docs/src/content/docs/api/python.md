---
title: Python API
description: Synchronous curl_cffi-shaped navigation with a live JavaScript Page.
---

```python
import brimp
```

The Python binding is synchronous and in process. It borrows familiar request
names from `curl_cffi.requests`, but returns a live `Page` rather than a
detached response.

## Module request helpers

```python
brimp.request(method, url, **options) -> Page
brimp.get(url, **options) -> Page
brimp.head(url, **options) -> Page
brimp.options(url, **options) -> Page
brimp.delete(url, **options) -> Page
brimp.post(url, data=None, json=None, **options) -> Page
brimp.put(url, data=None, **options) -> Page
brimp.patch(url, data=None, **options) -> Page
```

Each helper creates a private Session. The returned Page owns that Session, so
close it explicitly or use a context manager:

```python
with brimp.get("https://example.com") as page:
    print(page.status_code, page.evaluate("document.title"))
```

## Session

```python
brimp.Session(
    *,
    headers=None,
    cookies=None,
    params=None,
    auth=None,
    timeout=30,
    allow_redirects=True,
    max_redirects=30,
    proxy=None,
    referer=None,
    default_encoding="utf-8",
    persona_json=None,
    ca_bundle=None,
    enable_worker=False,
    enable_streaming_networking=False,
    enable_canvas=False,
    enable_webgl=False,
    enable_webgpu=False,
    enable_webaudio=False,
    enable_webaudio_output=False,
    storage_path=None,
    storage_quota_bytes=None,
    pool_size=None,
)
```

A Session owns a browser context, shared cookies, the direct connection pool,
the coherent persona, request defaults, and a bounded pool of native Pages.
`pool_size=None` resolves to twice `os.cpu_count()` (falling back to two). The
pool grows lazily, and requests block when every Page is leased. Call-specific
values override Session defaults. Pages can run concurrently from separate
Python threads.

`ca_bundle` adds trusted PEM roots; certificate and hostname verification
cannot be disabled. Heavy browser subsystems and persistent storage are
Session configuration because they shape every Page it creates.

### Session request helpers

Session exposes the same `request` and verb helpers as the module. Each call
leases and navigates a live Page while sharing Session cookies and direct
connections.

```python
with brimp.Session(pool_size=8, headers={"X-Agent": "brimp"}) as session:
    with session.post("https://example.com/login", json={"name": "agent"}) as page:
        page.click("#continue")
```

### `Session.new_page()`

```python
session.new_page(proxy=None) -> Page
```

Leases an un-navigated Page. Its direct, HTTP, SOCKS5, or SOCKS5H proxy is
immutable and applies to the main request and every resource created by that
document. Available Pages are reused only for the same proxy scope; an idle
incompatible slot may be closed and replaced.

## Page navigation

```python
page.request(
    method,
    url,
    *,
    params=None,
    data=None,
    content=None,
    json=None,
    headers=None,
    cookies=None,
    auth=None,
    timeout=None,
    allow_redirects=None,
    max_redirects=None,
    referer=None,
    default_encoding=None,
    multipart=None,
) -> Page
```

Page also provides all standard verb helpers. Navigation replaces the current
document and latest response metadata, then returns the same Page.

- `params` accepts mappings or ordered pairs and supports repeated values.
- mapping/pair `data` is form-urlencoded; string and bytes-like `data` remain
  accepted for curl_cffi compatibility.
- `content` is the preferred raw string/bytes-like body.
- `json` produces compact UTF-8 JSON.
- only one of `data`, `content`, `json`, and `multipart` may be supplied.
- `auth` supports only `(username, password)` HTTP Basic Auth.
- `timeout` is a positive total rendered-navigation deadline.
- redirects follow by default and produce bodyless ordered `history` entries.
- GET and HEAD reject request bodies.

`User-Agent` and `Accept-Language` are owned by the coherent Session persona and
cannot be overridden by headers.

### Multipart

```python
multipart = brimp.Multipart()
multipart.addpart(
    name="attachment",
    local_path="image.png",
    filename="upload.png",
    content_type="image/png",
)
page.post(url, data=None, multipart=multipart)
```

Each part uses exactly one of `data` or `local_path`. Uploads are buffered and
limited to 64 MiB. Brimp follows curl_cffi's explicit multipart direction and
does not implement Requests' ambiguous `files=` parameter.

## Latest navigation state

The Page directly exposes:

| Member | Meaning |
| --- | --- |
| `status_code`, `reason`, `url`, `headers` | Final main-response metadata. |
| `content` | Original decoded-transfer bytes from the final response. |
| `text`, `encoding`, `default_encoding` | Decoded body and mutable/fallback encoding controls. |
| `html` | Current live DOM serialization, or `None` for a non-HTML response. |
| `cookies` | Cookies set by the final response hop. |
| `elapsed`, `ok` | Navigation seconds and status below 400. |
| `last_request` | Final sent method, URL, headers, and body. |
| `history`, `redirect_count` | Ordered redirect records and their count. |
| `http_version` | Negotiated main-response HTTP version. |
| `downloaded_bytes`, `uploaded_bytes`, `header_bytes` | libcurl transfer sizes for the final hop. |

`json(**kwargs)`, `iter_content()`, `iter_lines()`, and `raise_for_status()`
operate on the original latest body. `HTTPError.page` refers to the Page that
raised it.

## Browser operations

```python
page.evaluate(expression)
page.screenshot(path=None, full_page=False)
page.extract(content_selector=None, remove_images=False, language=None, debug=False)
page.hover(selector)
page.click(selector)
page.type(selector, text)
page.tap(selector)
```

These operate on the live document. In particular, `page.html` reflects DOM
changes made by evaluation, scripts, and input after navigation.

## Cookies

`Session.cookies` is a native domain/path/secure-aware mutable mapping.
Responses update it at every redirect hop. It provides mapping access plus
`set()`, `delete()`, `clear()`, and `get_dict()`. Access by name raises
`CookieConflict` when multiple scoped cookies share that name.

## Lifecycle and errors

For a Page leased from an explicit Session, `Page.reset()` clears its document,
JavaScript realm, workers, sockets, streams, tasks, and navigation metadata,
then returns the native Page to the pool. The released wrapper is invalid and
raises `PageReleased` if reused. `with page` performs that reset and return even
when its body raises.

`Page.close()` permanently discards its native Page instead; the Session creates
a replacement lazily. A Page returned by a module helper owns a private Session,
so its context closes that Session. `Session.close()` closes available and
leased Pages and wakes threads waiting for a pool slot.

Errors derive from `RequestError`. Specialized types cover connection, proxy,
TLS, timeout, redirect-limit, invalid input/URL/header/proxy, cookie conflict,
HTTP status, JavaScript, and closed-object failures.

See the package support matrix for intentional curl_cffi incompatibilities,
including the absence of per-request fingerprints, `verify=False`, streaming
top-level responses, arbitrary curl options, retries, caching, and an async
facade.
