---
title: Python tutorial
description: Read live HTML, send requests, keep cookies, edit the DOM and configure identity.
---

**You'll build:** a small script that requests a page, changes its DOM and saves
the result. Each example is a complete program.

**Before you start:** install the [Python package](/install/#python)
and start the [local tutorial server](/quick-start/#1-start-the-example-site).

## 1. Compare the response with the live page

```python
import brimp

with brimp.Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    page.raise_for_status()
    print("Waiting for JavaScript" in page.text)
    print(page.evaluate("document.querySelector('h1').textContent"))
```

You should see `True` and `Hello from JavaScript`. The response still contains
the original HTML; page scripts have changed the live DOM.

## 2. Send query parameters and read JSON

Use Python's `urlencode` for query strings. Request headers belong in the
session constructor or the individual request.

```python
from urllib.parse import urlencode
import brimp

query = urlencode({"q": "browser", "tag": ["agents", "javascript"]}, doseq=True)
with brimp.Session(headers={"X-Client": "brimp-example"}) as session:
    page = session.get(
        f"http://127.0.0.1:8765/echo?{query}",
        headers={"X-Request-ID": "example-1"},
    )
    print(page.json()["query"])
```

The server returns `q` and both `tag` values. `page.json()` reads the HTTP response,
not the DOM.

## 3. Send forms, JSON or raw bytes

```python
import brimp

with brimp.Session() as session:
    page = session.post("http://127.0.0.1:8765/echo", data={"name": "Ada"})
    print(page.json())
```

Expect `{'method': 'POST', 'body': 'name=Ada'}`. Use `json=` for JSON and
`content=` or string/bytes `data=` for a raw payload:

```python
import brimp

with brimp.Session() as session:
    page = session.post(
        "http://127.0.0.1:8765/echo", json={"name": "Ada", "active": True}
    )
    print(page.json()["body"])
    page.post("http://127.0.0.1:8765/echo", content=b"raw bytes")
    print(page.json()["body"])
    page.post("http://127.0.0.1:8765/echo", data="already=encoded")
    print(page.json()["body"])
```

The output is compact JSON, `raw bytes`, then `already=encoded`. Brimp chooses
`application/json`, `application/octet-stream`, or
`application/x-www-form-urlencoded` for the corresponding input. An explicit
session or request `Content-Type` takes precedence.

For forms, use pairs such as `data=[("tag", "a"), ("tag", "b")]` for repeated
fields. Mapping values use `urlencode` without expanding lists, matching curl_cffi.
Strings are UTF-8 encoded; byte payloads are preserved, including embedded zeros.

Non-`None` `json` takes precedence over `content`, then `data`. `json=None` omits
the JSON body. The existing `body=` option accepts raw strings/bytes and cannot
be combined with the other body arguments. File-like objects, upload iterators,
`files=` and `multipart=` remain unsupported. GET/HEAD navigation still rejects
request bodies.

These body arguments also work with module helpers, `brimp.AsyncSession`, and
`page.stream()`. Response streaming buffers the upload body before sending it.

## 4. Keep cookies between requests

```python
import brimp

with brimp.Session() as session:
    session.set_cookie("http://127.0.0.1:8765/", "experiment", "rendered")
    session.get("http://127.0.0.1:8765/")
    second = session.get("http://127.0.0.1:8765/echo")
    print(second.json()["cookie"])
    print([cookie["name"] for cookie in session.cookies])
```

The echoed header contains `experiment=rendered`. `session.cookies` is a snapshot;
use `set_cookie()` or `clear_cookies()` to change the jar. Sharing cookies across
requests in one session does not persist them to disk.

## 5. Fill the form and save the edited HTML

```python
from pathlib import Path
import brimp

with brimp.Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    page.fill("#name", "Ada")
    page.click("button[type=submit]")
    heading = page.query_selector("h1")
    if heading is not None:
        with heading:
            heading.set_text_content("Edited by Python")
    print(page.evaluate("document.querySelector('#result').textContent"))
    Path("example.html").write_text(page.html(), encoding="utf-8")
```

Expect `Hello, Ada`. Open `example.html` to inspect the serialized DOM. Handles
support context managers; navigation makes handles from the previous page realm
stale. Input targets DOM elements without coordinate hit-testing.

## 6. Set browser identity and request timeouts

```python
import brimp

with brimp.Session(languages=["en-US", "en"], timeout=10.0) as session:
    page = session.get("http://127.0.0.1:8765/", timeout=5.0)
    print(page.evaluate("navigator.languages"))
```

Expect `['en-US', 'en']`. Identity options are `user_agent`, `languages` and
`platform`; arbitrary request headers cannot override profile-owned identity
headers. Pass a [captured or partial profile](/guides/profiles/) with `brimp.Session(profile=...)`.

Next: [extract and stream content](/examples/fetch/), or browse the
[generated Python API](/reference/python/).
