# Node support matrix

| API | Tested behavior |
| --- | --- |
| `get(url, options)` | Creates a temporary session, returns a detached Response, and closes native resources. |
| `createSession(options)` | Creates a browser context whose pages share cookies. |
| `Session.newPage({ proxy })` | Creates an independently concurrent page with a direct, HTTP, SOCKS5, or SOCKS5H document network scope. |
| `Page.get(url, options)` | Performs a GET navigation with query parameters, merged headers/cookies, timeout, and `AbortSignal` cancellation. |
| `Response.statusCode`, `reason`, `url`, `headers` | Exposes final main-response metadata without throwing for HTTP error statuses. |
| `Response.content` / `text` / `html` | Exposes original bytes, decoded response text, and the post-JavaScript DOM. |
| `Response.json()` / `raiseForStatus()` | Decodes JSON and explicitly throws `HTTPError` for 4xx/5xx responses. |
| `Page.evaluate`, `screenshot`, `extract` | Operates on that page's live document. |
| `Page.click`, `hover`, `type`, `tap` | Sends trusted human input to that page. |
| `Page.close()` | Closes one page and its transport scope. |
| `Session.close()` | Closes resources and is idempotent. |

Session headers and method headers are merged, with method values taking
precedence. `User-Agent` and `Accept-Language` remain persona-owned. Session and
method cookies are inserted into the shared browser-managed jar before
navigation. Response cookies remain on `Response.cookies` and persist natively.

Failures derive from `BrimpError`: `ConnectionError`, `Timeout`,
`TooManyRedirects`, `InvalidRequest`, `InvalidURL`, `HTTPError`, and
`JavaScriptError`. The initial API supports GET only.
