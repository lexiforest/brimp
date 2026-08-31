# Python support matrix

| API | Tested behavior |
| --- | --- |
| `get(url, **options)` | Creates a temporary Session, returns a detached Response, and closes native resources. |
| `Session(...)` | Creates a browser context whose pages share cookies; heavy browser subsystems are absent unless enabled. |
| `Session.new_page(proxy=None)` | Creates an independently concurrent page with a direct, HTTP, SOCKS5, or SOCKS5H document network scope. |
| `Page.get(url, params=None, headers=None, cookies=None, timeout=30)` | Performs a GET navigation with a clean per-navigation JavaScript realm. |
| `Response.status_code`, `reason`, `url`, `headers` | Exposes final main-response metadata without raising for HTTP error statuses. |
| `Response.content` / `text` | Exposes the original final HTTP response bytes and decoded text. |
| `Response.html` | Exposes serialized post-JavaScript DOM for HTML responses and `None` otherwise. |
| `Response.json()` / `raise_for_status()` | Decodes JSON and explicitly raises `HTTPError` for 4xx/5xx responses. |
| `Page.evaluate(source)` | Returns JSON-compatible Python values and rejects unsupported JavaScript values. |
| `Page.screenshot(path=None, full_page=False)` | Returns PNG bytes and optionally writes the same bytes to a path. |
| `Page.extract(content_selector=None, remove_images=False, language=None, debug=False)` | Runs pinned Defuddle extraction against the live DOM and returns content, Markdown, and metadata. |
| `Page.click`, `hover`, `type`, `tap` | Sends trusted human input to that page. |
| `Page.close()` / context manager | Closes one page and its transport scope. |
| `Session.close()` / context manager | Closes native resources deterministically and is idempotent. |

Session headers and method headers are merged, with method values taking
precedence. `User-Agent` and `Accept-Language` remain persona-owned. Session and
method cookies are inserted into the shared browser-managed jar. Response
cookies remain available on `Response.cookies` and persist natively in the jar.

The initial API deliberately supports GET only. Pages can run concurrently from
separate Python threads. A page's proxy is immutable for its lifetime.

Failures derive from `BrimpError`: `ConnectionError`, `Timeout`,
`TooManyRedirects`, `InvalidRequest`, `InvalidURL`, `HTTPError`, and
`JavaScriptError`. Uncaught website scripts do not fail navigation; explicit
`Page.evaluate()` exceptions raise `JavaScriptError`.
