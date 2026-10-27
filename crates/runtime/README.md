# Independent DOM runtime

This library composes [`brimp-jsc`](../jsc/README.md),
[`brimp-lexbor`](../lexbor/README.md), and [`brimp-net`](../net/README.md).
Those crates own engine bindings, DOM arenas, and transport respectively; this
crate owns browser DOM identity, Web APIs, sessions, plugins and scheduling.
The public in-process API is `runtime::native::{Session, NativePage, JSHandle,
ElementHandle}`. Session owns pages, cookies and its network dispatcher.
Python and Node packages are in [`bindings/`](../../bindings/README.md).

Each page owns its JSC realm and canonical Lexbor DOM on one thread. Commands
carry owned values; GC roots, DOM pointers, and Lexbor parsers stay on that thread.
Detached DOM nodes retain identity. Navigation invalidates external JS/DOM
handles; disposal and closure release roots and cancel pending operations.

The core supports HTML/fragment parsing, selectors, serialization, DOM mutation,
live collections, namespace attributes, form validation/reset/association, events,
MutationObserver, inline CSS and non-rendering stylesheet editing. Blocking scripts suspend the parser; async/deferred scripts,
modules, timers and cancellation use the page task loop. Promise jobs,
queueMicrotask and observer delivery share the engine microtask queue. Mutation
records from the parser are delivered at script boundaries.

Document lifecycle states, live Range content operations, FileReader, XPath,
Shadow DOM and structured cloning are core. Web Streams use the pinned
web-streams-polyfill implementation. WebCrypto shares curl-impersonate’s BoringSSL for SHA
digests, HMAC, AES-GCM/CBC/CTR/KW, RSA signing/OAEP, ECDSA/ECDH, HKDF and PBKDF2.
See [core coverage](../../docs/src/content/docs/web-platform/index.md) for supported formats and limits.

Networking includes redirects, session cookies, proxies, authentication,
Fetch/XHR, WebSocket, EventSource, URL/encoding/body primitives, typed events,
request/response/auth routes, retained bodies, cache controls, offline mode,
latency, and bounded streams. HTTP and WebSocket transport live in `brimp-net`; resource policy remains in
the runtime, and WebSocket connectors are separate from HTTP loaders.
Response interception buffers at most 32 MiB. Streams use a 256 KiB consumer
queue and replay paused chunks exactly once; retained response bodies have
separate configurable limits. Buffered download delay is not bandwidth control.

Browser subsystem plugins are enabled by default as ordered JavaScript installers.
An explicit plugin list replaces that set; an empty list selects core only.
See [plugin behavior and host callbacks](../../docs/src/content/docs/guides/plugins.md). Native
rendering, fonts, image decoding, GPU/audio engines and worker
execution are absent. Screen and viewport properties use configured profile
values; geometry objects perform value arithmetic without measuring layout.
Layout, computed style, XML parsing and coordinate actions are unsupported.

## Build and verify

Set `BRIMP_JSC_LIB_DIR` to a directory containing
`JavaScriptCore.framework/JavaScriptCore` on macOS, `JavaScriptCore.lib` on Windows,
or `libJavaScriptCore.so` on Linux. The local macOS default is the adjacent
WebKit release directory. `BRIMP_CURL_LIB_DIR` selects curl-impersonate (default
`/usr/local/lib`). Lexbor builds from the pinned source and license/provenance
under [`brimp-lexbor/vendor/lexbor`](../lexbor/vendor/lexbor/UPSTREAM.md).
The curl SDK must export the BoringSSL WebCrypto interface declared in
`curl-impersonate/webcrypto.h`; no separate crypto library is built by Cargo.

On macOS, a locally built WebKit framework can retain the absolute system
JavaScriptCore install name. Set `DYLD_FRAMEWORK_PATH="$BRIMP_JSC_LIB_DIR"` when
running tests or applications to select that SDK at runtime; link-time search
paths alone do not override its install name. For the default adjacent build,
use `export DYLD_FRAMEWORK_PATH="$PWD/../WebKit/WebKitBuild/Release"`.
`scripts/prepare_native.py` exports this setting for downloaded macOS SDKs.

```sh
cargo test -p brimp-runtime
python3 scripts/test_lexbor_boundary.py
cargo clippy -p brimp-runtime -p brimp-python -p brimp-node --all-targets -- -D warnings
```

The sanitizer script directly exercises the C boundary with ASan/UBSan. Tests
use local servers; they do not require public websites. Runtime tests target the
supported DOM, networking, and plugin contract.


Diagnostics and logical clocks are explicit session options. See
[Diagnostics and time](../../docs/src/content/docs/guides/diagnostics.md). Idle page owners and native
Promise waits use command/task/host wakeups and timer deadlines. Low-level
selector/network-idle cancellation checks and WebSocket I/O retain bounded polls;
these are not per-idle-page polling loops.
