# Node support matrix

| API | Tested behavior |
| --- | --- |
| `get(url, options)` | Creates a temporary session, returns a detached Response, and closes native resources. |
| `createSession(options)` | Creates one asynchronous native page session; heavy browser subsystems are absent unless explicitly enabled. |
| `Session.get(url, options)` | Performs a GET navigation with query parameters, merged headers/cookies, timeout, and `AbortSignal` cancellation. |
| `Response.statusCode`, `reason`, `url`, `headers` | Exposes final main-response metadata without throwing for HTTP error statuses. |
| `Response.content` / `text` / `html` | Exposes original bytes, decoded response text, and the post-JavaScript DOM. |
| `Response.json()` / `raiseForStatus()` | Decodes JSON and explicitly throws `HTTPError` for 4xx/5xx responses. |
| `Session.evaluate(source)` | Returns JSON-compatible JavaScript values and rejects unsupported values. |
| `Session.screenshot(options)` | Returns a PNG `Buffer` and optionally writes it to a path. |
| `Session.close()` | Closes resources and is idempotent. |

Session headers and method headers are merged, with method values taking
precedence. `User-Agent` and `Accept-Language` remain persona-owned. Session and
method cookies are sent together; response cookies update the Session mapping.

Failures derive from `BrimpError`: `ConnectionError`, `Timeout`,
`TooManyRedirects`, `InvalidRequest`, `InvalidURL`, `HTTPError`, and
`JavaScriptError`. The initial API supports GET only.
