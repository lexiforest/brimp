# Brimp Python binding

Brimp exposes a synchronous, Requests-style API backed by the shared native
browser runtime. It runs in-process and does not start a CDP server.

```python
import brimp

response = brimp.get("https://example.com")
print(response.status_code)
print(response.text)  # original HTTP response
print(response.html)  # DOM after JavaScript
```

Persistent cookies, connection reuse, JavaScript evaluation, and screenshots
are available through a context-managed Session:

```python
with brimp.Session() as session:
    response = session.get("https://example.com", timeout=30)
    response.raise_for_status()
    print(session.evaluate("document.title"))
    session.screenshot("page.png", full_page=True)
```

See `SUPPORT.md` for the exact tested surface.
