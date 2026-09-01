---
title: Python examples
description: curl_cffi-shaped navigation with live pages, extraction, state, and screenshots.
---

## One request, then interact

```python
import brimp

with brimp.get("https://example.com") as page:
    print(page.status_code)
    print(page.text)  # original response body
    print(page.html)  # live DOM
    page.click("a.more")
    print(page.html)
```

## JSON, forms, and raw bodies

```python
with brimp.Session(pool_size=8, auth=("agent", "secret")) as session:
    with session.post(
        "https://example.com/form",
        data={"name": "Luke", "tag": ["a", "b"]},
    ) as form:
        form.raise_for_status()

    with session.post(
        "https://example.com/api",
        json={"name": "Luke"},
    ) as api:
        print(api.json())

    with session.put(
        "https://example.com/blob",
        content=b"exact bytes",
    ) as upload:
        upload.raise_for_status()
```

## Explicit live page

```python
with brimp.Session() as session:
    with session.new_page(proxy="socks5h://127.0.0.1:1080") as page:
        page.get("https://example.com", params={"q": ["one", "two"]})
        page.type("#search", "browser")
        page.click("button[type=submit]")
        print(page.evaluate("document.title"))
        page.screenshot("result.png", full_page=True)
```

## Bounded Page reuse

```python
with brimp.Session(pool_size=8) as session:
    with session.get("https://example.com/one") as page:
        first = page.content

    # The first native Page has been reset and is available for this lease.
    with session.get("https://example.com/two") as page:
        second = page.content
```

Without `with`, the Page remains leased until explicitly returned:

```python
session = brimp.Session(pool_size=8)
page = session.get("https://example.com")
try:
    consume(page.content)
finally:
    page.reset()
```

`page.close()` discards a leased native Page instead of returning it.

## Redirect inspection

```python
with brimp.get("https://example.com/redirect") as page:
    print(page.url)
    for redirect in page.history:
        print(redirect.status_code, redirect.url, redirect.request.method)
```

## Multipart upload

```python
multipart = brimp.Multipart()
multipart.addpart(
    name="attachment",
    local_path="report.pdf",
    content_type="application/pdf",
)

with brimp.post("https://example.com/upload", multipart=multipart) as page:
    page.raise_for_status()
```

## Extraction and persona

```python
persona_json = open("persona/example.json").read()

with brimp.Session(persona_json=persona_json) as session:
    with session.get("https://example.com/article") as page:
        article = page.extract(content_selector="main", remove_images=True)
        print(article["contentMarkdown"])
```
