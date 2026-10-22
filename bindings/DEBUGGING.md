# Diagnostics and programmable time

See [runtime diagnostics](../TRACING.md) for trace fields, coverage, JSONL and
instrumentation effects. Both bindings expose the same owner-thread operations.

```python
from brimp import Session, plugins
with Session(time_mode="logical", time_origin=1700000000000,
             diagnostics={"path": "trace.jsonl", "include": ["Document.prototype"]},
             plugins=[plugins.scheduling(), plugins.geometry()]) as session:
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
