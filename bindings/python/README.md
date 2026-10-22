# Python

See [native binding setup and current scope](../README.md).

```python
from brimp import Session

with Session(headers={"X-Client": "native-example"}) as session:
    page = session.get("https://example.com")
    print(page.status_code, page.url)
    with page.query_selector("h1") as heading:
        heading.set_text_content("Edited in the DOM")
        print(heading.text_content())
    print(page.evaluate("Promise.resolve(document.title)"))
```
