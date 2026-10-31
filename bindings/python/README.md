# Python

See the [Python tutorial](https://docs.brimp.ai/examples/python/)
and [development setup](https://docs.brimp.ai/development/).

Browser subsystem plugins are enabled by default. An explicit `plugins` list
replaces them; `brimp.Session(plugins=[])` selects core only. Use `brimp.plugins.browser()`
to extend or customize the default set.

```python
import brimp

with brimp.Session(headers={"X-Client": "native-example"}) as session:
    page = session.get("https://example.com")
    page.raise_for_status()
    print(page.status_code, page.url)
    with page.query_selector("h1") as heading:
        heading.set_text_content("Edited in the DOM")
        print(heading.text_content())
    print(page.evaluate("Promise.resolve(document.title)"))
```
