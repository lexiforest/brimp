---
title: Python examples
description: Requests-style navigation, extraction, state, screenshots, and personas.
---

## Read rendered content

`Response.text` is the original HTTP response text. `Response.html` is the live
DOM serialized after scripts execute.

```python
import brimp

response = brimp.get("https://example.com")
response.raise_for_status()

print("original bytes:", len(response.content))
print("rendered HTML:", response.html)
```

For JSON responses, use the familiar `json()` helper:

```python
data = brimp.get("https://api.example.com/status").json()
print(data)
```

## Query parameters and headers

```python
import brimp

with brimp.Session() as session:
    session.headers["X-Client"] = "brimp-example"
    page = session.new_page()
    response = page.get(
        "https://example.com/search",
        params={"q": "headless browser", "tag": ["agents", "javascript"]},
        headers={"X-Request-ID": "example-1"},
    )
    print(response.url)
```

`User-Agent` and `Accept-Language` belong to the browser persona and cannot be
overridden as arbitrary request headers.

## Keep cookies between navigations

```python
import brimp

with brimp.Session() as session:
    session.cookies["experiment"] = "rendered"
    page = session.new_page()
    page.get("https://example.com/first")
    second = page.get("https://example.com/second")
    print(second.cookies)
```

## Evaluate and capture a screenshot

```python
from pathlib import Path
import brimp

with brimp.Session() as session:
    page = session.new_page()
    page.get("https://example.com")
    summary = page.evaluate("""
      ({
        title: document.title,
        links: document.querySelectorAll('a').length,
        text: document.body.textContent.trim()
      })
    """)
    png = page.screenshot(full_page=True)

print(summary)
Path("example.png").write_bytes(png)
```

Evaluation results must be JSON-compatible.

## Load a persona

```python
from pathlib import Path
import brimp

persona_json = Path("persona/example.json").read_text()

with brimp.Session(persona_json=persona_json) as session:
    page = session.new_page()
    page.get("https://example.com")
    print(page.evaluate("navigator.userAgent"))
```

See the [Python API](/api/python/) for the complete public surface.
