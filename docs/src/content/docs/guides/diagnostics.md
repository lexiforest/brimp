---
title: Diagnostics and time
description: Inspect browser API usage and control the page clock.
---

Use diagnostics to discover which browser APIs a script touches. Use logical time
to advance timers without waiting for wall-clock delays. Both are opt-in session
settings shared by Python, Node and the CLI.

For a runnable walkthrough, start with [Debug page scripts](/examples/debugging/).
This page describes configuration, record retention, tracing limits and clock behavior.

Diagnostics are opt-in per session. They are installed in every realm before
page scripts, after plugins, and remain active across navigation. This is a
tracing facility; it does not provide interactive debugger stepping.

```python
from brimp import Session
with Session(diagnostics={"path": "trace.jsonl", "include": ["Navigator.prototype", "Reflect"],
                          "capacity": 4096, "stack": True}) as session:
    page = session.new_page()
    page.evaluate("navigator.userAgent; Reflect.has(navigator, 'webdriver'); null")
    batch = page.take_diagnostics()
```

```javascript
const session = await createSession({diagnostics: {
  path: 'trace.jsonl', include: ['Navigator.prototype', 'Reflect'], stack: true
}});
const page = await session.newPage();
await page.evaluate("navigator.userAgent; Reflect.has(navigator, 'webdriver'); null");
const batch = await page.takeDiagnostics();
await session.close();
```

## Configuration and retained records

`path` is optional. With a path, complete JSONL records are appended as they are
produced; without it, records remain in the bounded page-owned ring. `include`
contains path prefixes; an empty list includes every instrumented path.
`capacity` defaults to 4096 and must be 1..1000000. `takeDiagnostics` / `take_diagnostics` consumes the ring
and its dropped-record count, but never truncates the file. A file-open failure
fails page initialization. A later write failure is reported in `writeError`
(Python `write_error`), disables file writes, and leaves bounded retrieval usable.
Multiple page owners serialize file writes. Stack capture is disabled by default.

`peekDiagnostics()` / `peek_diagnostics()` returns the same batch without draining
it. `summarizeDiagnostics()` / `summarize_diagnostics()` groups retained records by
layer, path, kind, and known reflection target/property, with counts and synchronous
throws. It also leaves the buffer and dropped count intact. Entries are sorted and
omit volatile sequence/page/realm IDs and timestamps; they combine retained
observations across realms. `retained` is the number of summarized records, not a
lifetime total. Filters and ring loss also apply to summaries. API wrappers and
native dispatch remain separate layers so callers need not count them as separate
browser dependencies.

All retrieval modes include `coverage.enabled`, `coverage.mode`, `coverage.include`, and an explicit
list of `coverage.limitations`. An empty disabled trace is distinguishable from
an enabled trace with no retained records. A thrown operation does not necessarily
mean unsupported behavior, and successful operations may use configured mocks.

The CLI exposes the same summary with `--trace-report dependencies.json`, usable
with or without `--trace trace.jsonl`. Reports are also written after ordinary
script errors, preserving the failure exit code; initialization failure or an
expired/cancelled operation can prevent retrieval. See the
[script-debugging tutorial](/examples/debugging/).

## Trace records

Records contain a monotonic per-page sequence, page/realm IDs, real elapsed
milliseconds, kind, path, outcome/type, optional stack, and bounded details.
Reflection details include the property name and the target path when known.
Arguments include at most eight primitive summaries (strings capped at 256
characters); object/function contents are never traversed. Native operations
report argument count. Ring overflow drops the oldest record and increments the
reported counter. Navigation changes the realm ID without resetting the sequence.

## Coverage

`diagnostics.mode` selects `api` (the default) or `recursive`. The CLI equivalent
is `--trace-mode recursive`, together with `--trace` or `--trace-report`.

### Recursive environment tracing

```javascript
const session = await createSession({diagnostics: {mode: 'recursive'}});
const page = await session.newPage();
await page.evaluate("void navigator.missing; document.body.dataset.state = 'ready'; null");
const summary = await page.summarizeDiagnostics();
await session.close();
```

Python accepts `Session(diagnostics={"mode": "recursive"})`. Rust exposes
`DiagnosticsOptions { mode: TraceMode::Recursive, ..Default::default() }`.

Recursive mode follows configurable browser/plugin roots and the `window`,
`self`, `top`, `parent`, and `globalThis` aliases through stable, realm-local
proxies. Nested objects, function results, constructor results and arguments
delivered to callbacks are traced. Reads (including absent properties), writes,
`in`, deletion, enumeration, descriptors, prototype/extensibility operations,
calls and construction produce records. Native dispatch remains a separate layer.
Engine intrinsics remain unwrapped; this includes direct `eval`.

Records have `details.layer: "recursive"`, a target path and, for property
operations, a property name. Symbols use their descriptive string; distinct
symbols with the same description are grouped together. Stable wrappers use the
first observed path for aliases; browser prototypes have canonical names such as
`Node.prototype`. Bound functions use a `.bound` suffix. Paths and property names
are capped at 512 characters. A `get` with outcome `undefined` can mean either an
absent member or a member whose value is undefined.

DOM receivers and arguments are unwrapped at browser implementation boundaries.
Callbacks re-enter tracing, including event listeners, observer callbacks and
Promise handlers. Listener adapters are cached so removal uses the same identity.
Private parser and input controllers run with raw globals. Setup and argument
logging do not enumerate object contents or invoke their getters.

This is environment tracing, not instrumentation of all JavaScript operations.
Raw aliases, local objects, bare missing global identifiers, browser implementation
internals, and reads through roots defined after installation that bypass the
window proxy are outside the boundary. Page-supplied objects returned unchanged
by APIs retain their original identity. Fixed properties/descriptors and
non-extensible prototypes can require raw references under JavaScript's proxy
rules; binary buffers/views also remain raw to preserve native interop. These
escapes emit `kind: "coverage"`, `outcome: "raw"`, with `details.note`. Filters
and ring limits apply to these notes too. Summaries count them as observations;
inspect the batch or JSONL for their explanations.

Proxy identity, global descriptors, reflection, timing and stacks differ from
untraced execution. In particular, retained raw aliases and proxied references
need not compare equal. Keep tracing disabled for performance measurements.

### Default API tracing

- Native DOM/platform operations crossing the Rust dispatch boundary, including
  whether the operation returned or threw. Paths begin with `native.`.
- Configurable browser prototype getters/setters/methods, global browser
  functions and constructors, including constructors installed by plugins.
- Selected `Object`/`Reflect` reflection operations and `Function.prototype.toString`.
- Existing page console, error, mutation, lifecycle and network subscriptions
  remain independent of diagnostics.

API tracing wraps function/accessor descriptors and therefore can change
function identity, reflection results, timing and stack contents. It is a debug
mode, not an undetectable execution mode. It does not proxy `window` or serialize
arbitrary page objects. Plain data-property reads, arbitrary missing properties,
engine-internal property accesses, and APIs installed after the diagnostic
installer runs are not comprehensively observed. Engine-wide interception would
require separate JSC instrumentation; this implementation does not claim it.

Keep tracing disabled for representative performance measurements. Use path
filters to reduce log volume. Plugins continue to declare mock/unavailable
behavior independently of whether their calls are traced.

## Logical time

```python
from brimp import Session, plugins
with Session(time_mode="logical", time_origin=1700000000000,
             diagnostics={"path": "trace.jsonl", "include": ["Document.prototype"]}) as session:
    page = session.new_page()
    page.evaluate("globalThis.times=[]; setTimeout(()=>times.push(Date.now()), 50); null")
    page.advance_time(50, max_tasks=10000)  # milliseconds, including in Python
    assert page.evaluate("times") == [1700000000050]
    records = page.take_diagnostics()
```

```javascript
const session = await createSession({clock: {mode: 'logical', epochMs: 1700000000000}});
const page = await session.newPage();
await page.evaluate('globalThis.times=[]; setTimeout(()=>times.push(Date.now()),50);null');
await page.advanceTime(50, {maxTasks: 10000});
console.log(await page.evaluate('times'));
await session.close();
```

System time is the default. Logical time begins at elapsed zero, with Unix epoch
zero unless an origin is supplied. Each new realm resets to the configured origin.
Timers, intervals, Date construction/Date.now, performance.now/timeOrigin and
requestAnimationFrame timestamps use the realm clock. The native scheduler never
changes system time or network, operation and cancellation deadlines.

`advance_time` / `advanceTime` runs due callbacks chronologically through the
requested nonnegative duration, including callbacks scheduled by callbacks.
Microtasks are checkpointed between tasks. `run_pending_tasks` /
`runPendingTasks` processes currently ready tasks without advancing logical time.
Both return `{now, tasks}`; `now` is elapsed logical/system milliseconds. The
positive task budget is capped at one million. Exhaustion raises an exception
at the budget boundary and leaves the clock at the last processed task, so an
interval cannot force unbounded advancement. A task's JavaScript execution also
remains subject to the real operation timeout.

While `evaluate` awaits a Promise in logical mode, an otherwise idle loop advances
to the next timer automatically. This allows synchronous Python evaluation of
`new Promise(resolve => setTimeout(resolve, 5000))` without a separate thread to
advance the clock. Async plugin installation uses the same rule. An unreturned
timer is advanced explicitly by the caller. External host/network completions
remain real events, so logical time alone does not make live-network races
reproducible; use supplied responses/routes for controlled fixtures. Advancing
logical time does not wait for all outstanding network activity.

The owner thread parks until a command, posted task, host completion/notification
or timer deadline wakes it. Native Promise waits use the same wakeups. HTTP
transport uses the brimp-net curl-multi coordinator; HTTP resource policy and
WebSocket connection policy are separate interfaces.
