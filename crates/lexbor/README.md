# brimp-lexbor

Safe, thread-bound Rust ownership for vendored Lexbor 3.0.0. This crate owns the
C shim, static native build, document arenas, parser suspension, node operations,
selectors, serialization and inline CSS declarations. Raw FFI is private.

Nodes carry an arena identity and every operation validates ownership. Detached
nodes remain alive until their document arena is released; parsers retain their
document. Browser NodeId mappings and JavaScript wrappers live in brimp-runtime.

```sh
cargo test -p brimp-lexbor
python3 scripts/test_lexbor_boundary.py
```

These commands run from the workspace root. Only Rust and a C compiler are needed
for the crate; the sanitizer script also needs Python and Clang. No JSC, curl,
Node or Python SDK is linked. An external Rust project can depend directly on
this crate by path. See [upstream provenance](vendor/lexbor/UPSTREAM.md) for
source version, local patches and licensing.
