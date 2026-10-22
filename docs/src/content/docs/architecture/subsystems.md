---
title: Core APIs and plugins
---

Brimp combines JavaScriptCore, a canonical Lexbor DOM, and curl-impersonate.
Common web APIs are available in every page. Specialized capabilities use explicit
plugins, with none enabled by default.

## Core coverage

| Area | Supported behavior | Limits |
| --- | --- | --- |
| DOM | HTML/fragment parsing, selectors, mutation, serialization, collections, forms, traversal, XPath, MutationObserver and Shadow DOM | No rendering/layout; XPath iterator mutation invalidation and some Shadow DOM features are unsupported |
| Lifecycle and ranges | readyState/readystatechange, DOMContentLoaded/load, live Range boundaries, clone/extract/delete/insert/surround contents and contextual fragments | No range geometry |
| Files and values | Blob/File, asynchronous FileReader, URL/text encoding, structuredClone and message channels | No XML parsing, XMLSerializer or CryptoKey cloning |
| Scripts and events | Classic/module scripts, parser suspension, timers, microtasks, DOM events and direct-node input | No coordinate hit-testing or browser Worker execution |
| Network | Fetch/XHR, cookies, redirects, proxies, auth, WebSocket, EventSource, routes and native streaming bodies | No WebTransport; response interception buffers at most 32 MiB |
| Streams | Readable/Writable/Transform streams, BYOB readers, closure, cancellation and backpressure via web-streams-polyfill | Unread fetch data has a 256 KiB native queue; an unread tee/clone branch can accumulate data |
| CSS/SVG | Lexbor inline declarations and generic SVG elements | No CSSStyleSheet/CSSRule editing, adopted sheets, cascade, computed style or specialized SVG interfaces |

DOM wrappers retain node identity, including detached nodes, until realm teardown.
Navigation invalidates external handles. Each page owns its JavaScript realm and
DOM arena on one owner thread.

## Core WebCrypto

The same vendored OpenSSL backend serves Python and Node. CryptoKey material and
usage/extractability policy live in native realm state; JavaScript wrappers hold
opaque handles. SubtleCrypto operations return Promises and copy binary inputs.

| Algorithms | Operations |
| --- | --- |
| SHA-1/256/384/512 | Digest |
| HMAC | Key generation/import/export, signing and verification |
| AES-GCM/CBC/CTR | 128/192/256-bit keys, encryption/decryption and key wrapping |
| AES-KW | Key wrapping/unwrapping |
| RSASSA-PKCS1-v1_5, RSA-PSS | Key generation/import/export, signing and verification |
| RSA-OAEP | Encryption/decryption and key wrapping, including labels |
| ECDSA | P-256/P-384/P-521 signatures |
| ECDH | Shared-secret and key derivation on matching curves |
| HKDF, PBKDF2 | Bit and key derivation |

RSA/EC keys support SPKI, PKCS8 and JWK; EC public keys also support raw points.
Symmetric keys support raw and JWK formats; HKDF/PBKDF2 base keys use raw import.
Ed25519/X25519 and CryptoKey structured cloning are unsupported. Expensive native
operations occupy the page owner thread. The complete algorithm and parameter
contract is in [core web coverage](https://github.com/lexiforest/brimp/blob/main/bindings/CORE_WEB.md).

## Bundled plugins

| Plugin | Behavior and limits |
| --- | --- |
| Storage | Realm-local Storage, fake-indexeddb transactions and Cache subset; resets on navigation |
| Host storage | Async hostStorage with atomic origin-partitioned batches; session memory by default, configurable backend for durability |
| Defuddle | DOM/attribute/inline-style extraction required by page.extract() |
| Canvas | State and fixed-color readback; no rasterization, fonts or image decoding |
| Services | Realm-local locks and configurable navigator/device/permission mocks |
| Geometry | Points, rectangles, quads and 2D matrices; no DOM measurement |
| CSS Typed OM | Inline value objects and attributeStyleMap; no cascade or compound calc algebra |
| Scheduling | Prioritized tasks, cancellation and zero-budget idle callbacks |
| Graphics | WebGL state and optional WebGPU buffer mocks; no shader execution or GPU rendering |
| Audio | AudioBuffer and graph/lifecycle state; no processing, decoding or output |

Inspect installer names, capabilities, limits, defaults and lifetimes from the host:

```python
from brimp import plugins
print(plugins.catalog())
```

```javascript
import { pluginCatalog } from 'brimp';
console.log(pluginCatalog());
```

Discovery installs nothing in a page. Select `plugins.host_storage()` in Python or
`hostStoragePlugin()` in Node for data shared by pages of the same origin within a
session. A `backend_factory` / `backendFactory` supplies custom atomic transactions
and optional cleanup. The SQLite example supplies durable JSON storage. This API
is an explicit asynchronous extension; it does not implement browser IndexedDB or
localStorage persistence.

## Custom plugins

Installers receive `{global, host, options}`, run in session order before document
scripts, and may return Promises and disposers. `host.call` returns a Promise for
registered handlers. Values are JSON-like or copied binary buffers; nodes, pointers
and functions stay in the realm. Ordered subscriptions carry host notifications.
Navigation and closure cancel pending calls and dispose realm resources.

Host setup/dispose factories own session state independently of realms. Trusted
handler context includes session, page, realm, URL and origin. Use that origin
for storage partitioning and coordinate shared backend transactions. Host errors
retain diagnostic details in the embedding application; page code receives a
generic OperationError.

See the [plugin contract and examples](https://github.com/lexiforest/brimp/blob/main/bindings/PLUGINS.md)
for file loading, options, callbacks, cancellation and storage backends.
