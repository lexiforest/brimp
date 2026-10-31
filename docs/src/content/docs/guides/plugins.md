---
title: Plugins
description: Choose plugins, load an installer, and manage host callbacks and state.
---

Use a plugin to add an API to page JavaScript or connect it to a function in your
Python or Node.js application. Article extraction is a bundled plugin; a lookup
against your own database is a custom one.

Start with the [plugin tutorial](/examples/plugins/) for a complete host callback
example. This guide covers selection, configuration, state, and cleanup. Browser
API behavior is listed in the [browser plugin reference](/web-platform/plugins/).

## Choose the plugins for a session

A normal session loads the default browser plugins. Supplying `plugins` replaces
that whole set. To add an article extractor while retaining the defaults:

```python
import brimp

with brimp.Session(plugins=[*brimp.plugins.browser(), brimp.plugins.extraction()]) as session:
    page = session.get("http://127.0.0.1:8765/")
    print(page.extract()["contentMarkdown"])
```

The example uses the [tutorial server](/quick-start/#1-start-the-example-site)
and prints Markdown containing `Hello from JavaScript`.

In Node.js, use `browserPlugins()` to obtain the default set:

```js
import { createSession, browserPlugins, extractionPlugin } from '@brimp/brimp';

const session = await createSession({plugins: [...browserPlugins(), extractionPlugin()]});
try {
  const page = await session.get('http://127.0.0.1:8765/');
  console.log((await page.extract()).contentMarkdown);
} finally {
  await session.close();
}
```

Use `plugins=[]` in Python or `plugins: []` in Node.js to select core APIs only.
Rust uses `bundled::browser()` for the defaults. On the CLI, repeated `--plugin`
flags select the complete set; `--no-plugins` selects core only. Markdown and
extraction JSON output also load the extractor.

## Find an installer or option

`brimp.plugins.catalog()` in Python and `pluginCatalog()` in Node.js list the bundled
installers, capabilities, limits, default activation, and options. Reading the
catalog does not create a page. Options marked `hostOnly`, such as a storage
backend factory, stay in your application.

For signatures, use the [Python plugin API](/reference/python/plugins/) or
[Node.js API](/reference/node/). For what a browser method actually does, use the
[browser plugin reference](/web-platform/plugins/).

## Load an installer from a file

An **installer** is a function that Brimp calls before a document's scripts. It
receives the page global object, plugin options, a browser profile, and a `host`
object for calling registered application functions. It runs again whenever
navigation creates a fresh JavaScript environment, or realm.

Use `brimp.Plugin.from_file()` in Python or `Plugin.fromFile()` in Node.js. The
checkout's [realm-library.js](https://github.com/lexiforest/brimp/blob/main/examples/plugins/realm-library.js)
installs a text normalizer. Run this example from the repository root:

```python
import brimp

plugin = brimp.Plugin.from_file(
    "examples/plugins/realm-library.js",
    name="text-tools",
    options={"form": "NFKC"},
)
with brimp.Session(plugins=[plugin]) as session:
    page = session.new_page()
    print(page.evaluate('textTools.normalize("Ａ")'))
```

This prints `A`. Plugin options are copied into each page. Keep private state
inside the installer function, a private field, or a WeakMap; export only the
APIs page scripts should use. Installers are trusted application code.

Write file-based installers as named function declarations so normal formatting
keeps them valid when loaded as function expressions. Bundle module dependencies
into the installer first, for example with esbuild. Installation errors include
the original file URL in host diagnostics; page stacks use a neutral source label.

## Keep state across navigation

State created inside an installer belongs to that document. Return a function,
or an object with `dispose()`, to clean it up when navigation or closure removes
the document. An async installer may return a Promise for its setup result.

Use `setup` for state shared by the session, such as a database connection or
origin-partitioned preference store. This synchronous function runs once per
session and returns `{handlers, dispose}` (a dict in Python). Its handlers extend
the configured handlers. Reusing a Plugin object in another session creates
independent setup state.

Session cleanup runs once, in reverse plugin order, after pages close. Failed
setup also disposes state already created. Node.js disposers may be async;
Python setup, handlers, and disposers are synchronous. A failed page installer
does not close its session.

Use `context.origin` for origin partitions, and reject `null`/`None` when an API
requires a non-opaque origin. This value and `context.url` come from the native
browsing context, not page JavaScript. `sessionId`/`session_id` identifies the
session, `pageId`/`page_id` survives navigation, and `realmId`/`realm_id` changes.
Pending invocations retain their captured context. Disposal of an old realm runs
before the next document's URL becomes current. Host state shared across pages
must be concurrency-safe in Rust; Node/Python adapters dispatch on their managed
host loop/thread.

The [Node.js](https://github.com/lexiforest/brimp/blob/main/examples/plugins/session-preferences.mjs)
and [Python](https://github.com/lexiforest/brimp/blob/main/examples/plugins/session_preferences.py)
examples keep preferences across reloads and partition them by origin. This is
an asynchronous application API. Keep its Promise visible to callers rather than
presenting it as a synchronous browser storage method.

## Call application functions and handle errors

Register the functions page code may call in `Plugin.handlers`. A call through
`host.call(name, ...args)` returns a Promise; unregistered names are rejected.
The [tutorial](/examples/plugins/#3-connect-page-code-to-a-host-handler) shows
this with a lookup function.

Arguments and results follow the [value conversion rules](/guides/values/).
Use `host.ref(value)` to pass a live object or function; the handler receives a
JSHandle tied to that page. `context.function(name)` exposes a registered
handler as a page function that returns a Promise.

Failed host calls reject in the page with
`DOMException("The operation failed.", "OperationError")`. Original adapter
errors, including Node stacks and Python tracebacks, go to the embedding
application's `page.on("error", ...)` channel with the plugin/handler name.
They are not sent to page console/error listeners. Subscribe before invoking
an API when host diagnostics are needed. Installation failures still return
the original host detail to the caller opening the page. For intentional public
failure data, return a structured result and translate it in the realm API.

### Callback lifetime and limits

Handlers receive `(context, ...args)`. Context exposes session/page/realm IDs, trusted URL/origin,
`cancelled`, and `emit(channel, value)`. JavaScript calls
`host.subscribe(channel, listener)` to receive ordered notifications and gets an
unsubscribe function. Notifications are bounded to 256 queued messages; pending
calls to 1024 per realm. Handle backpressure errors in the producer. Late results
from cancelled realms are discarded. Python uses one managed dispatcher per
session; Node uses its event loop and accepts Promise results. Callback reentry
into the same page rejects immediately, except for queued handle operations.
Those operations cannot call a host handler recursively.

## Choose storage lifetime

Browser localStorage, IndexedDB, and CacheStorage are available without a plugin.
Use [browser storage configuration](/web-platform/environment/#storage-and-shared-state)
when those APIs meet your needs. Choose host storage when page scripts need a
separate application API backed by your own storage implementation.

`hostStoragePlugin()` / `brimp.plugins.host_storage()` add an asynchronous
`hostStorage` API. Its default memory backend is owned by the session,
partitioned by the native origin, shared by that session's same-origin pages,
and survives navigation. Closing the Session discards it. Opaque origins reject.
It exposes get/set/delete/clear and transaction(operations), where a batch contains
`{type:'get',key}`, `{type:'set',key,value}`, `{type:'delete',key}` or `{type:'clear'}`.
A transaction returns results in order; set/clear return null and delete returns
a boolean. The default backend validates the whole batch and commits atomically.

```javascript
import { createSession, hostStoragePlugin } from '@brimp/brimp';

const session = await createSession({plugins: [hostStoragePlugin()]});
try {
  const page = await session.get('http://127.0.0.1:8765/');
  await page.evaluate('hostStorage.set("theme", "dark")');
  console.log(await page.evaluate('hostStorage.get("theme")'));
} finally {
  await session.close();
}
```

This prints `dark`. The example uses the same local tutorial server as above.

For a custom backend, pass Node `backendFactory` or Python `backend_factory`.
The synchronous factory runs once per Session and returns an object implementing
`transaction(origin, operations)` and optional `close()`. Node transactions/close
may be async; Python uses synchronous methods. Node serializes batches per origin,
and Python serializes backend access; cleanup waits for in-flight backend access.
The backend itself must guarantee atomic commit/rollback and coordinate other
sessions/processes if it shares durable state. Cancellation before a batch starts
prevents execution; cancellation during a committed backend transaction cannot
undo its writes. Late results are still discarded by the realm bridge.

[SQLiteStorage](https://github.com/lexiforest/brimp/blob/main/examples/plugins/sqlite_storage.py)
is a Python backend example with transactions and origin-separated records.
Supply its database path to persist JSON values to disk. The default memory
backend also accepts copied binary values.

## Overriding runtime behavior

A user plugin can replace much of the JavaScript-facing API. Its installer runs
in the page's JavaScript realm after core initialization and before document
scripts, with access to the global object. It can replace writable globals such
as `fetch`, patch prototype methods, replace configurable getters, and provide
alternative implementations backed by registered host handlers. Canvas backends
use the registration method described below.

There are three limits:

- **Property descriptors:** a non-writable, non-configurable property cannot be
  replaced. Inspect its descriptor before attempting an override.
- **Captured references:** core code may retain an original function internally.
  Replacing the public property does not redirect calls through those references.
- **Native ownership:** JavaScript patches cannot replace Lexbor, the native
  scheduler, or native key-policy checks. Replacing page `fetch` does not
  automatically change host `page.get()` navigation. A replacement crypto API
  can implement its own behavior, but cannot bypass the native checks on keys
  owned by the core implementation.

Installers run in descriptor order, so a later plugin can overwrite an earlier
plugin's changes where property descriptors permit. Each new realm runs the
installers again. Overrides share the page's realm and must preserve the browser
semantics that their consumers depend on.

## Registering a canvas backend

The installer context includes `canvas.register(type, factory)`. The factory runs
with the canvas as `this` and receives the context options. Return a context or
`null`. Context names are normalized to lowercase. Later registrations replace
earlier factories for new canvases; existing contexts keep their identity. Cleanup
removes a registration only if that factory is still current. Core returns null
for a different context type once a canvas has selected its context.

```javascript
import { createSession, Plugin } from '@brimp/brimp';

const plugin = new Plugin('custom-canvas', `({canvas}) => {
  canvas.register('custom', function(options) {
    return {canvas: this, options};
  });
}`);
const session = await createSession({plugins: [plugin]});
try {
  const page = await session.newPage();
  console.log(await page.evaluate(
    'document.createElement("canvas").getContext("custom", {answer: 42}).options.answer'
  ));
} finally {
  await session.close();
}
```

This prints `42`: the custom context receives the supplied options and keeps a
reference to its canvas.

For contributor decisions about core versus plugin ownership and Rust versus
JavaScript, see [Subsystem responsibilities](/architecture/subsystems/).
