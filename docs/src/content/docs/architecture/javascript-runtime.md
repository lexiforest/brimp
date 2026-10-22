---
title: JavaScriptCore integration
---

Each page's owner thread contains its JSC realm, GC roots, Lexbor arena, and task
queue. Typed commands carry copied values between native bindings and that thread.
JSHandle and ElementHandle refer to rooted values; navigation invalidates prior
realm tokens. Detached DOM nodes remain usable.

The parser pauses after parser-blocking script tokens, executes against the same
canonical tree, and resumes. Deferred/async scripts and the SWC module loader
retain their ordering/cancellation rules. Timers, microtasks, networking, and host
completions continue while native evaluation awaits a Promise. Parser and script
mutations reach MutationObserver; no layout information is fabricated.

The brimp-runtime crate owns browser policy, errors, requests, configuration,
extraction results, and typed events. brimp-jsc owns engine bindings and protected
values; brimp-net owns transport. brimp-lexbor exposes a safe arena API, with raw
FFI private to `crates/lexbor/src/lib.rs` and the C shim in
`crates/lexbor/native/lexbor.c`. Its arena is thread-confined and sanitizer-tested.
