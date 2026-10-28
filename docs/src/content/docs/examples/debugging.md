---
title: Debug page scripts
description: Call browser functions repeatedly and inspect their observed API dependencies.
---

Brimp's script harness uses the same Session/Page runtime as navigation. Load a
classic or bundled script, retain a function with a handle, and call it with
different inputs while its closure and DOM stay alive. Enable diagnostics to see
the instrumented browser APIs it uses. For configuration and precise limits, see
[Diagnostics and time](/guides/diagnostics/).

## Run the complete example

From a built source checkout, run either binding example:

```sh
node examples/harness/run.mjs
PYTHONPATH=bindings/python/python python3 examples/harness/run.py
```

Both load `examples/harness/page.js`, register network fixtures, click a button,
and invoke the same function twice. The results are:

```json
[
  {"text":"Hello Ada","calls":1,"clicks":1,"now":1005},
  {"text":"Hello Grace","calls":2,"clicks":1,"now":1010}
]
```

Each call advances a five-millisecond timer on the logical clock. Routes supply
the configuration response and reject other requests. The scripts print a
dependency report alongside the results and close all handles and the session.
See the [source examples](https://github.com/lexiforest/brimp/tree/main/examples/harness).

The equivalent single-call CLI workflow is:

```sh
brimp run examples/harness/page.js \
  --clock logical --epoch-ms 1000 \
  --routes examples/harness/routes.json \
  --actions examples/harness/actions.json \
  --eval 'compute("Ada")' \
  --trace trace.jsonl --trace-report dependencies.json
```

Standard output contains the result. `trace.jsonl` contains individual records;
`dependencies.json` groups retained records. Either diagnostic file can be
requested independently. Add `--overwrite` to replace existing files,
`--trace-include Document.prototype` to filter paths, or `--trace-stack` to
capture JavaScript stacks. Diagnostic outputs must use distinct file paths.

For ordinary script failures, the CLI writes the report before returning the
original failure exit code. Initialization failure, cancellation, or an expired
deadline can prevent report creation; JSONL already written remains available.

## Retain and call a function

The core sequence in Node is:

```javascript
const fn = await page.evaluateHandle(source + '\n;compute');
try {
  const result = await fn.call('(fn, input) => fn(input)', 'Ada');
  try { console.log(await result.jsonValue()); }
  finally { await result.dispose(); }
} finally { await fn.dispose(); }
```

In Python, handles are context managers:

```python
with page.evaluate_handle(source + '\n;compute') as fn:
    with fn.call('(fn, input) => fn(input)', 'Ada') as result:
        print(result.json_value())
```

These snippets assume an existing page and source string, as in the complete
examples. The appended expression selects the function in the same evaluation;
it does not move the function to the host. Calls await returned Promises. Arguments
use the [rich value bridge](/guides/values/), including nested handles and binary
values. Results remain handles until copied with `value()` or explicitly
serialized with `jsonValue` / `json_value`. Use `invoke` with an explicit receiver
when calling a retained object method.

Only select an expression available in that evaluation. A function hidden inside
a closed module or IIFE needs an explicit export from the bundle. There is no
automatic closure extraction or source rewriting. Navigation invalidates handles;
close the session to dispose all its pages and pending work.

## Inspect dependencies

| Node | Python | Effect |
| --- | --- | --- |
| `page.peekDiagnostics()` | `page.peek_diagnostics()` | Copy retained records without consuming them |
| `page.summarizeDiagnostics()` | `page.summarize_diagnostics()` | Group retained records without consuming them |
| `page.takeDiagnostics()` | `page.take_diagnostics()` | Consume retained records and reset the dropped count |

All three include `coverage`, `dropped`, and `writeError` (`write_error` in
Python). `coverage.mode` identifies `api` or `recursive` tracing. `coverage.enabled` distinguishes disabled tracing from an empty trace;
`coverage.include` contains path-prefix filters, and `coverage.limitations`
describes unobserved operations and instrumentation effects. Retrieval never
truncates JSONL output.

A summary contains `retained`, the number of records summarized, and sorted
`dependencies`. Each entry has a `layer`, `path`, `kind`, `count`, and `throws`.
Reflection entries also retain `target` and `property` when known. Recursive
summaries additionally contain `unresolved.reads` and `unresolved.operations`
with an owner (`core`, `profile`, `plugin:<name>`, or `unknown`), presence and
failure details. See [unresolved reads and failed operations](/guides/diagnostics/#unresolved-reads-and-failed-operations).
For example:

```json
{"layer":"api","path":"Document.prototype.querySelector","kind":"call","target":null,"property":null,"count":2,"throws":0}
```

The layers are `api`, `native`, `reflection`, and `recursive`. A single querySelector call
can produce both API and native records; treat these as two views of one action,
not two browser API calls. `throws` counts observed synchronous failures, not
later Promise rejections, and does not identify whether an API is unsupported.
Successful calls may still use configured mocks.

Reports cover only the retained, filtered window. Capacity defaults to 4096;
overflow discards the oldest records and increases `dropped`. Drain the buffer
after loading a bundle to begin a window around calls, as the examples do.
Navigation retains the page's diagnostic buffer. Summaries combine retained
observations across realms and omit sequence IDs, timestamps, arguments and
stacks, making equivalent observation counts easier to compare. They are not
execution-order or value replay logs.

## What a trace can establish

The default `api` mode covers native dispatch, configurable browser API
descriptors, and selected reflection operations. Use recursive mode for nested
browser/environment objects, missing reads, writes, `in`, deletion, symbols,
descriptors, prototypes, calls and construction:

```sh
brimp run examples/harness/page.js --clock logical --epoch-ms 1000 \
  --routes examples/harness/routes.json --actions examples/harness/actions.json \
  --eval 'compute("Ada")' --trace-mode recursive --trace-report recursive.json
```

In Node set `diagnostics: {mode: 'recursive'}`; in Python set
`diagnostics={"mode": "recursive"}`. Callback arguments and method results are
traced recursively, and stable wrappers preserve aliases within the traced
boundary. Recursive entries retain `target` and `property` in summaries.

Neither mode observes every JavaScript operation. Recursive mode leaves engine
intrinsics alone and cannot observe retained raw aliases, bare missing global
identifiers, local objects or browser implementation internals. JavaScript proxy
invariants require some fixed properties and non-extensible prototypes to remain
raw; binary buffers/views also retain native identity. These escapes produce
`coverage` records with an explanatory `details.note`. Inspect records for those
notes; summaries only count them. Filters and ring loss apply to notes too.

Paths use the first observed alias, with canonical browser prototype names.
Symbol names use their descriptions, so distinct symbols can share a report key.
An absent record is not evidence that a script never touched a member.
Instrumentation changes identity, global descriptors, reflection, timing and
stacks. Disable it for representative performance measurements. This facility
provides observations, not interactive stepping or an invisible debugger.

Logical time controls Date, performance and scheduled timers. It does not seed
Math.random or crypto randomness, serialize live network races, or control host
completion order. Use explicit routes and plugin fixtures for reproducible inputs;
real operation deadlines remain active. Specialized canvas, graphics and device
behavior still follows each [plugin's documented limits](/architecture/subsystems/).
