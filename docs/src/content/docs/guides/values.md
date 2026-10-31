---
title: Values and handles
description: Copy values, retain JavaScript references, and load WebAssembly.
---

Use `evaluate()` when you want a result copied into Python or Node.js. Use a
**handle** when you want to keep a JavaScript object in the page and work with it
later. A handle keeps that object alive until you dispose it, navigate the page,
or close the page.

The page's JavaScript environment is called a **realm**. Handles belong to that
realm; they cannot be reused after navigation or in another page. Python calls
are synchronous, while Node.js calls return Promises. Both wait for Promises
returned by page code.

| Page value | Python | Node |
| --- | --- | --- |
| undefined / null | `brimp.undefined` / `None` | `undefined` / `null` |
| Boolean, string, Number | bool, str, int/float | boolean, string, number |
| NaN, infinity, negative zero | preserved float values | preserved numbers |
| BigInt | `brimp.BigInt` (int subclass) | bigint |
| ArrayBuffer or typed-array/DataView bytes | bytes | Buffer |
| Arrays and plain objects | copied lists and string-keyed dicts | copied arrays and objects |
| Functions, DOM nodes, cyclic objects, class instances | explicit JSHandle | explicit JSHandle |

Python integers outside ±(2^53−1) become BigInt; use `brimp.BigInt(1)` to request a
small BigInt explicitly. Binary values copy only their visible byte range;
typed-array element types, backing-buffer identity and shared memory are not
transferred. Host bytes become a Uint8Array in the page. This is a copied-data
bridge, not a structured-clone or shared-memory interface. Unsupported copied
values raise errors; use references for identity and non-copyable values.

`evaluate_handle()` / `evaluateHandle()` creates a handle. `value()` copies the
held value; `json_value()` / `jsonValue()` converts it using `JSON.stringify`.
Use the latter when you want JSON's conversion rules. You can pass handles inside
arrays and objects when calling a page function.

```python
import brimp

with brimp.Session() as session:
    page = session.new_page()
    with page.evaluate_handle("function (n) { return this.base + n; }") as fn:
        with page.evaluate_handle("({base: 40n})") as receiver:
            with fn.invoke(brimp.BigInt(2), this=receiver) as result:
                assert result.value() == 42
            receiver.set_property("self", receiver)
            assert receiver.delete_property("self")
```

```js
const fn = await page.evaluateHandle('function (n) { return this.base + n; }');
const receiver = await page.evaluateHandle('({base: 40n})');
try {
  const result = await fn.invoke([2n], {thisArg: receiver});
  try { console.log(await result.value()); } finally { await result.dispose(); }
} finally { await fn.dispose(); await receiver.dispose(); }
```

`handle.call(source, ...args)` evaluates a function source and calls it with the
held value as its first argument. `invoke` calls the held function itself.
`property`, `set_property`/`setProperty`, `delete_property`/`deleteProperty`, and
`property_names`/`propertyNames` operate on the held object. Operations that
return handles retain their result separately, so each returned handle is owned
and disposed independently.

## Plugin references and functions

In an installer, `host.ref(value)` explicitly sends a JS value by reference:

```js
({global, host}) => {
  global.transform = (object, callback) =>
    host.call('transform', host.ref(object), host.ref(callback));
}
```

The Python/Node handler receives ordinary JSHandle objects. It can read or mutate
the object, invoke the callback, retain the handles beyond the invocation, or
return a handle to give the page its original object. The host owns received
handles and must dispose them when finished.

Handlers are registered through `Plugin.handlers`. `host.function(name)` creates
a page function for a registered handler; a host handler can return
`context.function(name)` to expose one in its result. These functions always
return Promises, preserve the plugin's handler allowlist, and expire with the
realm. They cannot serve as synchronous WASM imports; use a synchronous page
function for those imports.

Handle operations are queued on the owner thread. During a callback-originated
handle operation, host calls reject with `Reentry` to prevent circular waits.
Page evaluation, navigation and closure remain forbidden from their own host
callback. Python callbacks remain synchronous; Node callbacks may be async.
Lifecycle cancellation discards late results and releases the old realm's roots.

## WebAssembly

JSC runs the modules. The runtime services its deferred work on the owner thread,
including while waiting for Promises and while otherwise idle. Native engine
completion checks are bounded to 10 ms between host tasks; logical browser time
does not replace real engine/network scheduling.

`compile`, `instantiate`, imports/exports, Memory, Table and Global use the SDK's
native implementation. `compileStreaming` and `instantiateStreaming` accept a
Response or a Promise for one, require a successful accessible response with
`application/wasm` (without parameters), and consume the existing Fetch body.
Body errors, locked/used bodies and failed compilation reject their Promises.

The streaming APIs buffer the body before native asynchronous compilation;
they do not perform incremental compilation. This follows the
[WebAssembly Web API algorithm](https://webassembly.github.io/spec/web-api/#streaming-modules).
Closing/navigating a page cancels its realm work. Compilation proposals are
limited to the selected JSC SDK. Browser Workers are not added by WASM support.
