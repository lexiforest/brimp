# Brimp Python binding

Brimp combines a curl-impersonate transport with a live JavaScript browser
runtime. Its request vocabulary follows `curl_cffi.requests` where that maps
cleanly, but every navigation returns a live `Page` instead of a detached
response object.

```python
import brimp

with brimp.get("https://example.com") as page:
    print(page.status_code)
    print(page.text)  # original main-response body
    print(page.html)  # current live DOM
    page.click("#continue")
    print(page.html)  # includes DOM changes caused by the click
```

Use a Session to share cookies, connections, and a bounded pool of reusable
Pages. The pool defaults to twice the logical CPU count and grows lazily.
Session request helpers lease and navigate a Page; leaving its context resets
and returns the native Page to the pool.

```python
with brimp.Session(pool_size=8, headers={"X-Agent": "brimp"}) as session:
    with session.post(
        "https://example.com/login",
        data={"name": "agent"},
    ) as login:
        login.raise_for_status()
        login.type("#code", "123456")
        login.click("#submit")

    with session.new_page(proxy="socks5h://127.0.0.1:1080") as page:
        page.get("https://example.com", timeout=30)
        print(page.evaluate("document.title"))
        print(page.extract(content_selector="main")["contentMarkdown"])
```

`request()` and the `get`, `head`, `options`, `delete`, `post`, `put`, and
`patch` helpers accept query parameters, ordered headers, cookies, Basic Auth,
form `data`, raw `content`, JSON, redirect controls, a scalar navigation
timeout, a referrer, and buffered `Multipart` uploads.

The Page stores main-response metadata from its latest navigation:
`status_code`, `reason`, `url`, `headers`, `content`, `text`, `cookies`,
`elapsed`, `last_request`, `history`, `http_version`, `downloaded_bytes`,
`uploaded_bytes`, and `header_bytes`. A later navigation replaces those values
just as it replaces the document. `html` is different: it serializes the current
DOM each time it is read.

Module helpers create a private Session owned by the returned Page, so their
Page context closes that Session. A Page leased from an explicit Session is
reset and returned by its context or by `page.reset()`. `page.close()` discards
that native Page; the Session lazily replaces the slot when needed. Released
wrappers raise `PageReleased`.

Private test and enterprise roots can be trusted without disabling certificate
or hostname verification:

```python
with brimp.Session(ca_bundle="path/to/cacert.pem") as session:
    with session.get("https://internal.example") as page:
        print(page.status_code)
```

Persona JSON uses the versioned schema in [`../../persona/`](../../persona/README.md).
Persona-owned headers and the curl impersonation profile are coherent with the
JavaScript-visible environment and therefore cannot be changed per request.

Heavy browser subsystems remain Session configuration and are disabled by
default: `enable_worker`, `enable_streaming_networking`, `enable_canvas`,
`enable_webgl`, `enable_webgpu`, `enable_webaudio`, persistent storage, and
optional WebAudio system output.

See `SUPPORT.md` for the exact tested surface and intentional differences from
curl_cffi.

## Binary wheels

Release workflows produce self-contained CPython 3.10+ ABI3 wheels for
manylinux 2.28 x86-64/ARM64, macOS ARM64, and Windows x86-64. Source
distributions are intentionally omitted because their native dependency build
is not self-contained.
