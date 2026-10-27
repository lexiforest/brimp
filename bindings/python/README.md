# Python

See the [Python tutorial](https://docs.brimp.ai/examples/python/)
and [development setup](https://docs.brimp.ai/development/).

Browser subsystem plugins are enabled by default. An explicit `plugins` list
replaces them; `Session(plugins=[])` selects core only. Use `plugins.browser()`
to extend or customize the default set.

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
