---
title: Follow a page through the runtime
description: Observe script execution, handle identity and realm replacement.
---

This walkthrough follows one page from navigation through DOM mutation and a
second navigation. Start the [local tutorial server](/quick-start/#1-start-the-example-site).

## 1. See where a command goes

![Python, Node and the CLI submit commands to a page owner thread. Its JSC realm and Lexbor DOM stay together; transport and host callbacks return copied results through the task queue.](/diagrams/page-lifecycle.png)

The host API sends typed commands with owned values to the page's owner thread.
That thread owns the page's JavaScript environment (its realm), DOM tree, and
queued tasks. Native objects stay on that thread; host calls exchange values
and handles with it.

## 2. Observe page script execution

```python
import brimp

with brimp.Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    print("Waiting for JavaScript" in page.text)
    print(page.evaluate("document.querySelector('h1').textContent"))
```

Expect `True` and `Hello from JavaScript`. The HTML parser pauses for a blocking
script, executes it against the same DOM tree, then resumes. Deferred, async and
module scripts use the runtime's scheduling and cancellation rules.

## 3. Keep a detached node alive

```python
import brimp

with brimp.Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    heading = page.query_selector("h1")
    if heading is not None:
        with heading:
            heading.remove()
            print(heading.text_content())
```

The detached handle still prints `Hello from JavaScript`. A handle keeps its
JavaScript value alive; removing its DOM node does not invalidate that identity.

## 4. Navigate to a new realm

```python
import brimp

with brimp.Session() as session:
    page = session.get("http://127.0.0.1:8765/")
    heading = page.query_selector("h1")
    if heading is not None:
        page.get("http://127.0.0.1:8765/article")
        try:
            heading.text_content()
        except brimp.StaleHandle:
            print("The previous realm is gone")
```

Navigation invalidates old realm tokens, releases roots and cancels pending
realm work. Query the new page again to obtain fresh handles.

## 5. Await a Promise while work continues

```python
import brimp

with brimp.Session(time_mode="logical") as session:
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
| HTTP and WebSocket transport | [`brimp-net`](/development/network/) |

### Runtime source files

The Rust implementation keeps `Page` state, construction, realm reset and
destruction together in `crates/runtime/src/runtime/page.rs`. Its child modules
extend the same `Page` type:

| Module under `runtime/page/` | Responsibility |
| --- | --- |
| `navigation.rs` | Navigation, load state and response metadata |
| `network.rs` | Page fetches, streaming responses and WebSockets |
| `tasks.rs` | Task execution, timers, clock advancement and deadlines |
| `plugins.rs` | Plugin installation and disposal for each realm |
| `profile.rs` | Profile installation, identity updates and host reports |
| `options.rs` | Page configuration and its builder |
| `scripts.rs` | Script discovery, loading and execution order |

Host operations are implemented under `runtime/native/`:

| Module under `runtime/native/` | Responsibility |
| --- | --- |
| `session.rs` | Host session setup and managed page creation |
| `page.rs` | Host methods that submit work to a managed page |
| `handles.rs` | JS/DOM references, shared disposal and element operations |
| `options.rs` | Identity, session, request and file-upload configuration |
| `operations.rs` | Owner-thread command execution and promise completion |
| `streaming.rs` | Host response bodies, bounded buffering and backpressure |

`native/page.rs` operates through a `PageHandle`; the `Page` implementation above
owns the actual JavaScript realm and DOM. Handle roots and their release remain
with the owner-thread bridge.

Request policy lives in `runtime/request.rs`: buffered requests and shared cookie,
identity and redirect handling. Its child `request/streaming.rs` applies that
policy to callback-driven Fetch and EventSource streams inside the page. The host
streaming API lives in `native/streaming.rs`, where consumers pull response chunks
through `BodyStream`; it shares the request policy helpers.

Next: [subsystem responsibilities](/architecture/subsystems/).
