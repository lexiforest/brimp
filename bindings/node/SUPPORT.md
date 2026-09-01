# Node support matrix

| API | Tested behavior |
| --- | --- |
| module `request` and verb helpers | Create a private Session and return its live owning Page. |
| `createSession(options)` | Creates a browser context with shared cookies, transport, persona, and request defaults. |
| Session request/verb helpers | Create, navigate, and return a live Page. |
| `Session.newPage({ proxy })` | Creates an un-navigated Page with an immutable direct, HTTP, SOCKS5, or SOCKS5H scope. |
| Page request/verb helpers | Navigate in place and resolve to that same Page. |
| request inputs | Query parameters, merged headers/cookies, Basic Auth, referrer, buffered form/raw/JSON/multipart bodies, redirects, timeout, and AbortSignal. |
| Page HTTP state | Latest status, reason, URL, headers, original bytes/text, rendered HTML, cookies, elapsed seconds, final sent request, bodyless redirect history, HTTP version, and transfer byte counts. |
| browser operations | Evaluation, screenshots, extraction, and trusted click/hover/type/tap on the live document. |
| lifecycle | Page and Session close are idempotent; closing an owning module-helper Page closes its private Session. |

`User-Agent` and `Accept-Language` remain persona-owned. A Page proxy applies to
the complete document resource graph. Streaming top-level responses, arbitrary
curl options, request-level transport fingerprints, `verify: false`, retries,
and caching are intentionally not exposed by rendered navigation.

Failures derive from `RequestError`: connection, timeout, redirect-limit,
invalid request/URL, HTTP status, JavaScript, and closed-object errors.
