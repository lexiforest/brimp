---
title: Cryptography
description: Supported WebCrypto algorithms, key formats and execution limits.
---

WebCrypto is available in core without a plugin. Python and Node use the same
native backend. The table below lists implemented algorithms; other algorithms
reject with NotSupportedError.

`crypto.getRandomValues`, `randomUUID` and `crypto.subtle` are core. The same Rust
BoringSSL backend embedded in curl-impersonate serves both language bindings.
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

## Key policy and algorithm limits

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
key generation/derivation occupies the page owner thread. The runtime calls the narrow crypto interface exported by curl-impersonate,
sharing its BoringSSL implementation with TLS. BoringSSL notices are supplied
with curl-impersonate; the runtime does not duplicate them or build OpenSSL.

The `jose` fixture checks HS256 signing and verification. The `jose-extended`
fixture checks RSA/PSS/ECDSA JWTs and RSA-OAEP/ECDH-AES-KW JWE, including damaged
signature/tag rejection.
## Verification

The [runtime tests](https://github.com/lexiforest/brimp/tree/main/tests/runtime/environment/)
record the behavior behind this reference. Safari comparisons cover the recorded
cases; they do not establish complete browser conformance. The
[field ledger](https://github.com/lexiforest/brimp/blob/main/COVERAGE.yaml) tracks
individual decisions and evidence.
