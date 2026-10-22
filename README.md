# Brimp

**A JavaScript runtime for JavaScript from the browser.**

The DOM runtime combines **JavaScriptCore + Lexbor + curl-impersonate** behind
synchronous Python and asynchronous Node `Session`/`Page` APIs. It fetches pages,
runs scripts, maintains a live DOM, and exposes requests, handles, events,
interception, input, extraction, and streaming bodies. Rendering and layout are
outside its contract. Optional JavaScript plugins add memory storage, deterministic
Canvas mocks, services, Defuddle, or host storage; none are enabled by default.
Core APIs include live DOM ranges, FileReader, Web Streams, structured cloning,
XPath, Shadow DOM, and WebCrypto for signing, encryption, and key derivation.

```python
from brimp import Session

with Session() as session:
    page = session.get("https://example.com")
    heading = page.query_selector("h1")
    print(heading.text_content())
    heading.dispose()
```

```javascript
import { createSession } from 'brimp';
const session = await createSession();
try {
  const page = await session.get('https://example.com');
  console.log(await page.evaluate('document.title'));
} finally { await session.close(); }
```

See [native setup and API](bindings/README.md), [plugin behavior](bindings/PLUGINS.md),
[runtime architecture](crates/runtime/README.md), and [implementation progress](TODO.md).
Native package builds require matching JavaScriptCore and curl-impersonate libraries.
Lexbor and OpenSSL are pinned and built from vendored sources.

The reusable [`brimp-lexbor`](crates/lexbor/README.md) crate owns the vendored
parser and safe DOM arena API; it builds without JSC, curl or language SDKs.
[`brimp-jsc`](crates/jsc/README.md) owns the JavaScript engine and native callbacks.
`brimp-runtime` composes these with networking and browser policy; Python and Node
bindings depend on the runtime.

The [`brimp-net` crate](crates/net/README.md) supplies the runtime's curl-impersonate
HTTP transport through standard Rust HTTP types and streaming bodies. Browser
loading policy remains in the runtime; WebSocket connections use a separate connector.

[Diagnostics and programmable time](bindings/DEBUGGING.md) provide bounded API
traces, JSONL output and a controllable realm clock. Optional geometry, CSS Typed
OM, scheduling, graphics and audio plugins expand the API surface; their
[coverage table](bindings/PLUGINS.md) distinguishes functionality from mocks.
See [native packaging](bindings/PACKAGING.md) for relocatable macOS artifacts.


```sh
cargo test -p brimp-runtime
python3 scripts/test_lexbor_boundary.py
cargo clippy --workspace --all-targets -- -D warnings
```

Tests target supported behavior and local servers. See the [benchmarks](benchmark/native/)
for measured time and peak memory on the same host/profile; no universal speed or
browser-conformance claim is made. MIT licensed.

The [browser workload corpus](compatibility/README.md) runs 15 pinned applications,
libraries and environment checks against Brimp, jsdom, happy-dom and Playwright
Chromium using the same artifacts. Detector findings are observational and do not expand runtime
coverage. Reports distinguish compatibility failures from invalid reference runs.

Core DOM and crypto coverage: [supported web APIs](bindings/CORE_WEB.md).
