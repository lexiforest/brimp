---
title: Python API
description: Source-generated Python signatures, properties and plugin APIs.
---

[Open the generated Python API reference →](/reference/python/)

The reference is built from `bindings/python/python/brimp` with mkdocstrings.
It includes inherited request methods and the plugin module, without importing
the native extension during the documentation build.

For a runnable introduction, follow the [Python tutorial](/examples/python/).

Evaluation now copies undefined/null, BigInt, special numbers and binary buffers.
Use JSHandle for identity, functions and cyclic values. The
[value bridge guide](/guides/values/)
covers reference calls, plugin callbacks, ownership and WebAssembly loading.

## Browser profiles

`page.profile_contract()` and `page.profile_report()` describe the loaded interface definitions and report
defaulted values and capture provenance. See [browser capture profiles](/guides/profiles/).
