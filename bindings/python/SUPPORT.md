# Python support matrix

| API | Tested behavior |
| --- | --- |
| `get(url, **options)` | Creates a temporary Session, returns a detached Response, and closes native resources. |
| `Session(..., enable_worker=False, enable_streaming_networking=False, storage_path=None, storage_quota_bytes=None)` | Creates one synchronous native page session; heavy browser subsystems are absent unless explicitly enabled. |
| `Session.get(url, params=None, headers=None, cookies=None, timeout=30)` | Performs a GET navigation with a clean per-navigation JavaScript realm. |
| `Response.status_code`, `reason`, `url`, `headers` | Exposes final main-response metadata without raising for HTTP error statuses. |
| `Response.content` / `text` | Exposes the original final HTTP response bytes and decoded text. |
| `Response.html` | Exposes serialized post-JavaScript DOM for HTML responses and `None` otherwise. |
| `Response.json()` / `raise_for_status()` | Decodes JSON and explicitly raises `HTTPError` for 4xx/5xx responses. |
| `Session.evaluate(source)` | Returns JSON-compatible Python values and rejects unsupported JavaScript values. |
| `Session.screenshot(path=None, full_page=False)` | Returns PNG bytes and optionally writes the same bytes to a path. |
| `Session.close()` / context manager | Closes native resources deterministically and is idempotent. |

Session headers and method headers are merged, with method values taking
precedence. `User-Agent` and `Accept-Language` remain persona-owned. Session and
method cookies are sent with browser-managed cookies; response cookies update
the Session cookie mapping.

The initial API deliberately supports GET only. Sessions are sequential and
not thread-safe. POST bodies, streaming, multipart uploads, prepared requests,
transport adapters, per-request proxies, and an asynchronous Python facade are
not exposed.

Failures derive from `BrimpError`: `ConnectionError`, `Timeout`,
`TooManyRedirects`, `InvalidRequest`, `InvalidURL`, `HTTPError`, and
`JavaScriptError`. Uncaught website scripts do not fail navigation; explicit
`Session.evaluate()` exceptions raise `JavaScriptError`.
