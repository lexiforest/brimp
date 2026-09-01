# Python support matrix

| API | Tested behavior |
| --- | --- |
| `request()` and HTTP verb helpers | Available at module, Session, and Page levels; all return a live Page. |
| Module helpers | Create a private Session; closing the returned Page closes that Session. |
| `Session(...)` | Reuses its direct transport, owns the browser cookie jar and bounded Page pool, and supplies mergeable request defaults. `pool_size` defaults to `2 * os.cpu_count()`. |
| `Session.request()` / verbs | Lease, navigate, and return a live Page, blocking when every pool slot is leased. |
| `Session.new_page(proxy=None)` | Leases an un-navigated Page with an immutable direct, HTTP, SOCKS5, or SOCKS5H network scope. Pools never mix proxy scopes. |
| `Page.request()` / verbs | Navigate in place and return the same Page. GET/HEAD reject bodies. |
| `params`, `headers`, `cookies` | Encode repeated query values, merge headers case-insensitively, and insert cookies into the browser jar. |
| `data`, `content`, `json` | Send form-urlencoded, buffered raw, or compact UTF-8 JSON bodies with inferred content headers. |
| `Multipart` | Buffers explicit named data/path parts up to 64 MiB; Requests-style `files=` is not supported. |
| `auth`, `referer` | Support HTTP Basic Auth and a Referer shortcut. |
| redirects | Follow by default, honor `allow_redirects` and `max_redirects`, expose ordered bodyless history, and apply browser method/credential rules. |
| `timeout` | A positive scalar deadline for complete rendered navigation, not a curl connect/read tuple. |
| Page response fields | Latest `status_code`, `reason`, `url`, `headers`, original `content`/`text`, final-hop cookies, elapsed seconds, `last_request`, history, HTTP version, and transfer byte counts. |
| `Page.html` | Serializes the current live DOM for HTML navigation and is `None` for non-HTML main responses. |
| buffered body helpers | Mutable `encoding`, configurable fallback encoding, `json(**kwargs)`, `iter_content()`, `iter_lines()`, and `raise_for_status()`. |
| Session cookies | Domain/path/secure-aware native jar with mapping access and explicit set/delete/clear operations. |
| browser operations | `evaluate`, `screenshot`, `extract`, `click`, `hover`, `type`, and `tap` operate on the live Page. |
| lifecycle | A Session Page context resets and returns its lease; `Page.reset()` returns explicitly, `Page.close()` discards, and stale wrappers raise `PageReleased`. Module Pages close their private Session. |

`User-Agent` and `Accept-Language` remain persona-owned. A Page's proxy applies
to its main navigation, redirects, scripts, styles, images, Fetch, workers,
streams, and WebSockets.

Intentional non-compatibilities with curl_cffi:

- no public `Response`; Page is the navigation result and controller;
- no per-request impersonation, JA3/Akamai fields, default browser headers, or
  HTTP-version forcing because the Session persona owns one coherent transport
  and JavaScript identity;
- no `verify=False`; use Session `ca_bundle` for additional roots;
- no scheme-based `proxies` map, environment proxy discovery, client
  certificates, interface/DoH controls, or arbitrary curl options;
- no top-level streaming response/upload, callback, throttling, caching,
  automatic retry, or Python async facade.

Failures derive from `RequestError`: `ConnectionError`, `ProxyError`,
`SSLError`, `Timeout`, `TooManyRedirects`, invalid request/URL/header/proxy
errors, `CookieConflict`, `HTTPError`, `JavaScriptError`, `PageReleased`, and
`SessionClosed`.

## curl_cffi request parameter matrix

This matrix compares Brimp with the synchronous
[`curl_cffi.requests`](https://curl-cffi.readthedocs.io/en/latest/api.html#module-curl_cffi.requests)
request surface. “Session” means the setting belongs to a coherent browser
context; “Page” means it belongs to the page's immutable network scope.

| curl_cffi parameter | Brimp status | Brimp spelling or reason |
| --- | --- | --- |
| `method`, `url` | Supported | `request(method, url)` and verb helpers; all return Page. |
| `params` | Supported | Ordered pairs, mappings, repeated values, and Session defaults. |
| `data` | Supported | Form mappings/pairs or buffered string/bytes. |
| `json` | Supported | Compact UTF-8 JSON with inferred content type. |
| `headers` | Supported | Ordered, duplicate-preserving `Headers`; persona-owned fields are protected. |
| `cookies` | Supported | Scoped browser jar plus request cookies. |
| `auth` | Supported | Basic `(username, password)` only. |
| `timeout` | Renamed semantics | One positive rendered-navigation deadline, not a connect/read tuple. |
| `allow_redirects`, `max_redirects` | Supported | Browser rewrites, cookies, and credential stripping; covered by redirect tests. |
| `referer` | Supported | Also accepted through `headers`. |
| `default_encoding` | Supported | Fallback for Page `text`; `encoding` remains mutable. |
| `multipart` | Supported | Explicit buffered `Multipart`, limited to 64 MiB. |
| `files` | Intentionally unsupported | Use `Multipart`; avoids the ambiguous Requests compatibility shape. |
| `proxy` | Page-scoped | Accepted while creating a Page; immutable for its document and subresources. |
| `proxies`, `proxy_auth` | Intentionally unsupported | A rendered document cannot mix network identity by URL scheme; put credentials in the proxy URL. |
| `verify` | Session-scoped | Verification is always on; add roots with Session `ca_bundle`. |
| `impersonate`, `ja3`, `akamai`, `extra_fp`, `default_headers` | Session-scoped | Configure one coherent persona, not per-request fingerprints. |
| `http_version` | Session-owned | Reported on Page but not forceable per request. |
| `accept_encoding` | Renamed | Supply `Accept-Encoding` through coherent Session headers/persona policy. |
| `quote` | Intentionally unsupported | Pass the exact URL/query pairs to navigate. |
| `stream`, `content_callback`, `max_recv_speed` | Intentionally unsupported | Brimp must consume and render the main document before returning Page. |
| streaming/file-object request bodies | Deferred | Upload replay across 307/308 requires a native streaming design. |
| `curl_options`, `interface`, `doh_url`, `cert` | Intentionally unsupported | These need browser-scope policy or bypass browser invariants. |
| `discard_cookies`, adapters, cache, automatic retry | Intentionally unsupported | They conflict with browser jar/state semantics or require separate policy. |
| `AsyncSession`, `thread` | Deferred | Async live-Page ownership and cancellation require their own API. |
