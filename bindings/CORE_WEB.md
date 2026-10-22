# Common web core

Brimp owns the canonical DOM, events, script execution, network primitives,
structured cloning and crypto. These are available in both Node and Python
without plugins. Specialized capabilities are explicit plugins; see [PLUGINS.md](PLUGINS.md).

The browser corpus exercises input handlers/activation, token lists,
NodeIterator, attribute removal, XPath and non-rendering Shadow DOM. Shadow roots
have isolated canonical subtrees, named slots, connected roots and composed event
retargeting, including closed-root path filtering. Custom element definitions
read observedAttributes before construction, as required by Lit's initialization.
There is no CSS cascade, computed geometry or rendering. Manual slot assignment,
declarative shadow roots, adopted stylesheets and full focus delegation remain
outside this implementation. XPath uses the pinned `xpath` library; mutation
invalidation of XPath iterators is not yet implemented.

## Page exposure boundary

Each page executes in its own JavaScriptCore realm. The Node/Python host objects
are not installed there: `process`, `global`, `Buffer`, `require`, `module`,
`__filename` and `__dirname` are absent from the page global by default. Plugin
host calls cross the copied-value bridge, not a shared host object graph.

Script scheduling, currentScript control, module loading and frame-index
bookkeeping use native-held realm functions and private state throughout
navigation and module execution.
The bundled core-js shared store stays inside its bundle. Core and plugin APIs
share initialization-time source finalization; plugin file paths and raw host
exceptions remain host diagnostics. See [the plugin contract](PLUGINS.md).

There is no global-name filtering or freeze applied to application code. Pages
can create properties (even names resembling internals), replace mutable APIs,
and load their own dependencies. Trusted plugins control their intentional
public exports. This boundary does not claim full security isolation or
human-browser indistinguishability: engine behavior, module compilation,
unsupported APIs, optional mocks and network/interaction behavior remain
observable. Detector findings do not expand the core contract.

## Crypto

`crypto.getRandomValues`, `randomUUID` and `crypto.subtle` are core. The same Rust
vendored OpenSSL backend serves both language bindings.
Implemented SubtleCrypto operations are Promise-returning, copy BufferSource
inputs, and enforce CryptoKey usages and extractability. Key bytes and authoritative usage/extractability policy live in native realm state.
JavaScript CryptoKey wrappers hold opaque handles; generated keys never pass their
bytes through JavaScript. Keys are released when the realm is destroyed.

| Algorithm | Operations and formats |
| --- | --- |
| SHA-1, SHA-256, SHA-384, SHA-512 | digest |
| HMAC with those hashes | generate/import/export, sign/verify; raw and oct JWK |
| AES-GCM/CBC/CTR | 128/192/256-bit keys; generate/import/export, encrypt/decrypt, wrap/unwrap; raw and oct JWK |
| AES-KW | RFC 3394 key wrapping/unwrapping; raw and oct JWK |
| RSASSA-PKCS1-v1_5, RSA-PSS | generate/import/export, sign/verify; SPKI, PKCS8, RSA JWK |
| RSA-OAEP | generate/import/export, encrypt/decrypt, wrap/unwrap; hash-bound keys and labels |
| ECDSA | P-256/P-384/P-521; raw public points, SPKI, PKCS8 and EC JWK; browser-format signatures |
| ECDH | P-256/P-384/P-521; deriveBits/deriveKey with matching public peer keys |
| HKDF, PBKDF2 | raw non-extractable base keys; deriveBits/deriveKey |

AES-GCM accepts nonempty IVs and browser tag lengths (32, 64, 96, 104, 112,
120, 128 bits); callers must supply a unique IV per key. AES-CBC uses PKCS#7
padding. AES-CTR respects the requested 1–128 counter bits, including rollover,
and rejects inputs that would reuse a counter. RSA generation accepts
1024–16384-bit moduli. RSA JWK private imports require all CRT components; EC
raw public imports require uncompressed points. Derived/generated secret material
stays native unless explicitly exported. Native checks bind key type, algorithm,
usages, extractability and peer curve; modifying JS metadata does not change them.

Ed25519/X25519, CryptoKey structured cloning and full WebCrypto conformance are
not implemented. Unsupported algorithms reject with NotSupportedError. Crypto
uses native synchronous operations behind Promise-returning Web APIs; expensive
key generation/derivation occupies the page owner thread. OpenSSL is statically
built from the Cargo-pinned source, with packaged license notices; no additional
OpenSSL runtime SDK is required.

The `jose` fixture checks HS256 signing and verification. The `jose-extended`
fixture checks RSA/PSS/ECDSA JWTs and RSA-OAEP/ECDH-AES-KW JWE, including damaged
signature/tag rejection.

## DOM lifecycle, ranges, files and streams

Navigation exposes loading -> interactive -> complete through document.readyState
and readystatechange, with DOMContentLoaded before complete and load. Range has
cross-node text, clone/extract/delete contents, insertion, surrounding and
contextual fragments, with boundary updates for DOM mutations, character-data
edits, splitText and normalize. These are non-rendering operations; no range
rectangles or layout measurements are supplied.

FileReader supports asynchronous text, ArrayBuffer, binary-string and data-URL
reads, progress/lifecycle events, abort and reentrant reads. Data-URL reads encode
actual Blob bytes; Canvas mock output remains explicitly separate.

Web Streams use pinned `web-streams-polyfill`. The library owns readers/writers,
closure, errors, locking, piping,
backpressure and BYOB behavior. Fetch's byte stream releases native buffer credit
when a reader takes a chunk; unread network bytes stay bounded by the 256 KiB
queue. Response cloning tees the stream, so an unread clone branch can accumulate
buffered data as with the Streams tee algorithm. This is not a claim of complete
Fetch/body conformance.

## CSS and SVG

Core CSS support covers inline declarations parsed by Lexbor. The optional
Typed OM plugin provides inline value objects and attributeStyleMap operations.
Generic SVG elements participate in the DOM tree.

CSSStyleSheet construction, rule editing, document/shadow-root adoptedStyleSheets,
and live style/link sheet ownership are not implemented. Cascade, computed style,
style-dependent geometry, animation timelines and rendering are unsupported.
Specialized SVG interfaces, XML parsing and XMLSerializer are also unsupported.

## Structured cloning

`structuredClone` uses `core-js-pure` with the engine's ArrayBuffer transfer. It
preserves cycles and shared references in supported JavaScript values, including
Maps/Sets, dates and binary views. Functions, DOM nodes, CryptoKeys and unsupported
platform objects reject with DataCloneError. Message channels use this shared
clone implementation too.

The [compatibility corpus](../compatibility/README.md) records exact upstream
artifacts and Chromium comparisons. Detector findings, including CreepJS, remain
observational and do not define the product's API boundary.
