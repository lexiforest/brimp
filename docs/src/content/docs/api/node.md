---
title: Node API
description: Generated reference for @brimp/brimp.
---

[Open the generated Node API reference →](/reference/node/)

TypeDoc reads `bindings/node/index.d.ts` on each documentation build. Browse
session options, request options, page methods, event records, routes and plugins.

For runnable ESM examples, follow the [Node tutorial](/examples/node/).

Evaluation supports bigint, undefined, special numbers and binary buffers.
JSHandle supports nested reference arguments, property mutation and invocation
with an explicit receiver. See the
[value bridge guide](/guides/values/)
for plugin functions, ownership and WebAssembly loading.

## Browser profiles

`page.profileContract()` and `page.profileReport()` describe the loaded interface definitions and report
defaulted values and capture provenance. See [browser capture profiles](/guides/profiles/).
