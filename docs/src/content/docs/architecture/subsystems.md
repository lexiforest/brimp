---
title: Core APIs and plugins
---

Brimp combines JavaScriptCore, a canonical Lexbor DOM, and curl-impersonate.
DOM APIs and shared infrastructure live in core. Browser subsystem plugins are
installed by default, including their placeholders and synthetic behavior. An
explicit plugin list replaces the defaults; an empty list selects core only. CLI
Markdown/JSON extraction also selects Defuddle.

In this walkthrough, you will use core crypto, inspect plugin options and add a
host-backed lookup function to page JavaScript.

## 1. Use a core API without a plugin

```python
from brimp import Session

with Session() as session:
    page = session.new_page()
    length = page.evaluate("""
      crypto.subtle.digest('SHA-256', new TextEncoder().encode('hello'))
        .then(buffer => buffer.byteLength)
    """)
    print(length)
```

Expect `32`. WebCrypto belongs in core; it does not need an installer.

## 2. Choose a bundled plugin

Inspect capabilities without creating a page:

```python
from brimp import plugins
print(plugins.catalog())
```

Browser storage is a core API. For optional application-specific storage, use
`plugins.host_storage()` with a backend. Its [contract](/guides/plugins/) includes
a SQLite example. Core localStorage, CacheStorage and IndexedDB share origin state
within a session; `storage_path` optionally persists their committed contents.

## 3. Connect page code to a host handler

```python
from brimp import Session, Plugin

lookup = Plugin(
    name="lookup",
    source="""({global, host}) => {
      global.lookup = value => host.call('lookup', value);
      return () => { delete global.lookup; };
    }""",
    handlers={"lookup": lambda context, value: {"answer": value + 1}},
)
with Session(plugins=[lookup]) as session:
    page = session.new_page()
    print(page.evaluate("lookup(41)"))
```

Expect `{'answer': 42}`. The installer runs before document scripts and may return
a disposer. `host.call()` returns a Promise; Python's synchronous handler receives
a trusted `HostContext` and copied arguments. Node handlers may be asynchronous.

![Page JavaScript, DOM and queued work remain on the owner thread; a plugin host handler receives copied values and returns its result through the task queue.](/diagrams/page-lifecycle.png)

Host handlers can use realm-bound references; lifecycle/evaluation reentry and
recursive host calls from callback handle operations are rejected. Navigation cancels pending realm
calls and runs cleanup. For storage, partition by `context.origin`, not an origin
supplied by page code.

## 4. Check the boundary before adding a plugin

Core owns DOM APIs and shared runtime infrastructure. Browser subsystems such as
audio, graphics, RTC and device services belong in plugins, including their
placeholders and synthetic behavior. A real backend is not required for a plugin
to provide useful compatibility. Which plugins the default environment installs
is a separate decision from ownership.

The tables below describe current coverage. Constructor presence is not evidence
of a full graphics, audio or worker implementation, but those callable surfaces
can still be required by browser scripts. See the [ownership policy](/guides/plugins/#core-runtime-and-plugins)
before changing placement or exposure.

## Core coverage

| Area | Supported behavior | Limits |
| --- | --- | --- |
| DOM | HTML/fragment parsing, selectors, mutation, serialization, collections, forms, traversal, XPath, MutationObserver and Shadow DOM | No rendering/layout; XPath iterator mutation invalidation and some Shadow DOM features are unsupported |
| Lifecycle and ranges | readyState/readystatechange, DOMContentLoaded/load, live Range boundaries, clone/extract/delete/insert/surround contents and contextual fragments | No range geometry |
| Files and values | Blob/File, asynchronous FileReader, URL/text encoding, structuredClone and message channels | No XML parsing, XMLSerializer or CryptoKey cloning |
| Scripts and events | Classic/module scripts, parser suspension, timers, microtasks, DOM events and direct-node input | No coordinate hit-testing or browser Worker execution |
| Network | Fetch/XHR, cookies, redirects, proxies, auth, WebSocket, EventSource, routes and native streaming bodies | No WebTransport; response interception buffers at most 32 MiB |
| Streams | Readable/Writable/Transform streams, BYOB readers, closure, cancellation and backpressure via web-streams-polyfill | Unread fetch data has a 256 KiB native queue; an unread tee/clone branch can accumulate data |
| CSS/SVG | Lexbor inline declarations, style/media/supports rule editing, style/link sheets, constructed/adopted sheets and generic SVG elements | Partial CSS grammar; no cascade, computed style or specialized SVG interfaces |

DOM wrappers retain node identity, including detached nodes, until realm teardown.
Navigation invalidates external handles. Each page owns its JavaScript realm and
DOM arena on one owner thread.

## Core WebCrypto

The BoringSSL instance embedded in curl-impersonate serves both Python and Node. CryptoKey material and
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
contract is in [core web coverage](/web-platform/).

## Bundled plugins

| Plugin | Behavior and limits |
| --- | --- |
| Host storage | Async hostStorage with atomic origin-partitioned batches; session memory by default, configurable backend for durability |
| Defuddle | DOM/attribute/inline-style extraction required by page.extract() |
| Canvas | State, solid rectangle/putImageData painting and real PNG encoding; paths, images and text do not rasterize |
| Environment / Permissions / Devices | Navigator configuration, synthetic permissions and device-service placeholders |
| Media / Workers / Trusted Types | Media objects, worker shapes and policy placeholders; no media/worker execution or policy enforcement |
| Graphics | WebGL state/constants with profile vendor, renderer and extensions; legacy callable placeholders; optional WebGPU buffers; no shader execution or GPU rendering |
| Audio | AudioBuffer and graph/lifecycle state; no processing, decoding or output |

See the [generated Python plugin API](/reference/python/plugins/) and
[Node API](/reference/node/) for installer signatures. The
[plugin contract](/guides/plugins/)
covers file loading, notification subscriptions, per-session setup/disposal,
cancellation and storage backends.
