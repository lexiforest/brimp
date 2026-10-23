---
title: Follow a page through the runtime
description: Observe script execution, handle identity and realm replacement.
---

This walkthrough follows one page from navigation through DOM mutation and a
second navigation. Start the [local tutorial server](/quick-start/#1-start-the-example-site).

## 1. See where a command goes

![Python, Node and the CLI submit commands to a page owner thread. Its JSC realm and Lexbor DOM stay together; transport and host callbacks return copied results through the task queue.](/diagrams/page-lifecycle.png)

The host API sends typed commands with owned values to the page's owner thread.
That thread owns the JavaScriptCore realm, protected JS values, Lexbor DOM arena
and task queue. Native pointers stay on the owner thread.

## 2. Observe page script execution

```python
from brimp import Session

with Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    print("Waiting for JavaScript" in page.text)
    print(page.evaluate("document.querySelector('h1').textContent"))
```

Expect `True` and `Hello from JavaScript`. The HTML parser pauses for a blocking
script, executes it against the same DOM tree, then resumes. Deferred, async and
module scripts use the runtime's scheduling and cancellation rules.

## 3. Keep a detached node alive

```python
from brimp import Session

with Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    heading = page.query_selector("h1")
    if heading is not None:
        with heading:
            heading.remove()
            print(heading.text_content())
```

The detached handle still prints `Hello from JavaScript`. A handle keeps a rooted
JS value; removing its DOM node does not invalidate that identity.

## 4. Navigate to a new realm

```python
from brimp import Session, StaleHandle

with Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    heading = page.query_selector("h1")
    if heading is not None:
        page.get("http://127.0.0.1:8765/article")
        try:
            heading.text_content()
        except StaleHandle:
            print("The previous realm is gone")
```

Navigation invalidates old realm tokens, releases roots and cancels pending
realm work. Query the new page again to obtain fresh handles.

## 5. Await a Promise while work continues

```python
from brimp import Session

with Session(time_mode="logical") as session:
    page = session.new_page()
    print(page.evaluate("new Promise(resolve => setTimeout(() => resolve(42), 50))"))
```

Expect `42`. Evaluation pumps timers, networking and host completions while it
awaits a Promise. Logical mode can advance to the next timer automatically.
Promise callbacks, `queueMicrotask` and MutationObserver delivery share the
engine's microtask queue. Network and cancellation deadlines still use real time.

## Follow the implementation

| Responsibility | Crate |
| --- | --- |
| Browser policy, sessions, plugins and task scheduling | `brimp-runtime` |
| Engine bindings and protected values | `brimp-jsc` |
| Safe DOM arena; private C/FFI boundary | `brimp-lexbor` |
| HTTP and WebSocket transport | `brimp-net` |

Next: [choose core APIs or a plugin](/architecture/subsystems/).
