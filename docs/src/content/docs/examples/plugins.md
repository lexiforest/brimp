---
title: Add a plugin to a page
description: Use a bundled extractor and connect page JavaScript to a Python function.
---

You'll add article extraction to a session, then expose a Python lookup function
to page JavaScript. Install the [Python package](/install/#python)
and start the [tutorial server](/quick-start/#1-start-the-example-site) first.
Each example is a complete program.

## 1. Use a core API without a plugin

Core APIs are available in every page. For example, WebCrypto can hash text even
when the session selects no plugins:

```python
import brimp

with brimp.Session(plugins=[]) as session:
    page = session.new_page()
    length = page.evaluate("""
      crypto.subtle.digest('SHA-256', new TextEncoder().encode('hello'))
        .then(buffer => buffer.byteLength)
    """)
    print(length)
```

Expect `32`, the byte length of a SHA-256 digest.

## 2. Choose a bundled plugin

Article extraction uses the bundled Defuddle plugin. Add it to the usual browser
plugins, request a page, and extract its current document:

```python
import brimp

with brimp.Session(plugins=[*brimp.plugins.browser(), brimp.plugins.extraction()]) as session:
    page = session.get("http://127.0.0.1:8765/")
    print(page.extract()["contentMarkdown"])
```

The Markdown contains `Hello from JavaScript`. Supplying a plugin list replaces
the default set, so `brimp.plugins.browser()` keeps the normal browser APIs available.

## 3. Connect page code to a host handler

A custom plugin can expose an API backed by your application. Here, the
installer defines `lookup()` in the page and registers a Python function to
answer its calls:

```python
import brimp

lookup = brimp.Plugin(
    name="lookup",
    source="""({global, host}) => {
      global.lookup = value => host.call('lookup', value);
      return () => { delete global.lookup; };
    }""",
    handlers={"lookup": lambda context, value: {"answer": value + 1}},
)
with brimp.Session(plugins=[lookup]) as session:
    page = session.new_page()
    print(page.evaluate("lookup(41)"))
```

Expect `{'answer': 42}`. The installer runs before document scripts. Its `host`
object can call only the handlers you register. `host.call()` returns a Promise,
which `evaluate()` waits for before returning the copied result to Python.
The function returned by the installer removes the page API during cleanup.

Python handlers return synchronously; Node.js handlers may return Promises.
Use this pattern for application lookups or services. A handler cannot evaluate,
navigate, or close the page that is waiting for it.

Continue with the [plugin guide](/guides/plugins/) for file loading, options,
notifications and state that survives navigation. To check what bundled browser
APIs actually do, use the [browser plugin reference](/web-platform/plugins/).
