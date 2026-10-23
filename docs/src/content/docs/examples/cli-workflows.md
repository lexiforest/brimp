---
title: CLI workflows
description: Add input actions, fixture routes, custom plugins, streams and logical time to a CLI job.
---

These recipes build on the [CLI tutorial](/examples/cli/). Start the
[local example server](/quick-start/#1-start-the-example-site), then work in a
scratch directory. Save the named files before running their commands.

## 1. Fill a form with actions

Save this ordered action list as `actions.json`:

```json title="actions.json"
[
  {"action":"fill", "selector":"#name", "text":"Ada"},
  {"action":"click", "selector":"button[type=submit]"},
  {"action":"wait-selector", "selector":"#result"}
]
```

```sh
brimp fetch http://127.0.0.1:8765/ --actions actions.json \
  --eval 'document.querySelector("#result").textContent'
```

Expect `"Hello, Ada"`. Actions target DOM nodes, without visibility checks or
coordinate hit-testing. An element's presence does not imply it is visible.

Other actions are `focus`, `hover`, `tap`, `type`, `press`, `drag`, `upload`,
`evaluate`, `wait`, `wait-network-idle` and `advance-time`. Upload paths are
relative to the action file. Unknown operations or fields reject the input.

| Action | Fields in addition to `action` |
| --- | --- |
| `click`, `focus`, `hover`, `tap`, `wait-selector` | `selector` |
| `fill`, `type` | `selector`, `text` |
| `press` | `selector`, `key`, optional `modifiers` bitmask: Alt=1, Control=2, Meta=4, Shift=8 |
| `drag` | `source`, `destination` selectors |
| `upload` | `selector`, `files` |
| `evaluate` | `source` JavaScript |
| `wait` | `seconds` |
| `wait-network-idle` | `milliseconds`, default 500 |
| `advance-time` | `milliseconds` |

## 2. Supply a network fixture

Save `routes.json`:

```json title="routes.json"
[
  {
    "pattern":"http://127.0.0.1:8765/api/*",
    "stage":"request",
    "decision":{
      "action":"fulfill",
      "status":200,
      "headers":[["Content-Type","application/json"]],
      "body":"{\"source\":\"route\"}"
    }
  }
]
```

```sh
brimp fetch http://127.0.0.1:8765/ --routes routes.json \
  --eval 'fetch("/api/status").then(response => response.json())'
```

Expect `{"source":"route"}`. Rules are glob patterns, and the last matching
rule at a stage wins. A file response uses `"file":"fixture.json"` in place of
`body`; the path is relative to the routes file.

| Decision | Behavior |
| --- | --- |
| `block` | Abort; optional `message` |
| `fulfill` | Supply `status`, header pairs, and a text `body` or `file` |
| `continue` | Optional URL, method, headers, body and status overrides |

The stage defaults to `request`; `response` is also supported. Response-stage
continuation cannot change URL/method, and request-stage continuation cannot
change status. Supplied headers replace the routed header list. Response
interception buffers at most 32 MiB. Use `--auth USER:PASSWORD` for authentication
challenges; credentials are scoped to the initial target origin.

## 3. Install a bundled or custom plugin

Discover capabilities:

```sh
brimp plugins list
brimp plugins show storage
```

Opt into realm-local browser storage:

```sh
brimp run - --plugin storage <<'JS'
localStorage.setItem('theme', 'dark'); localStorage.getItem('theme')
JS
```

Expect `"dark"`. The plugin's state resets on navigation. CLI `host-storage`
instead supplies session-memory, origin-partitioned atomic batches; it has no
disk persistence and does not make browser localStorage durable.

For a custom installer, save `custom.js`:

```javascript title="custom.js"
({global, options}) => {
  global.answer = options.answer;
  return () => { delete global.answer; };
}
```

```sh
brimp run - --plugin ./custom.js --plugin-options 'custom={"answer":42}' <<'JS'
answer
JS
```

Expect `42`. File plugin names come from the file stem. Repeat `--plugin` to
install in argument order; duplicate names reject. Installers run before page
scripts, may return Promises, and may provide cleanup. They receive
`{global, host, options}`; custom Python/Node host handlers require the bindings.

Canvas, graphics and audio plugins retain their documented mock limits.
Install Canvas before graphics when combining them. Crypto is core.

## 4. Stream bytes to a file

```sh
brimp fetch http://127.0.0.1:8765/download --stream --format raw \
  --output streamed.bin --response-info response.json
```

Expect a 53,248-byte file. Streaming neither navigates nor runs page scripts;
it cannot accompany init/preparation scripts, actions, clock advancement or page
waits. Metadata contains status, URL and headers. The result file is published
only after the body finishes; stdout can contain partial data on failure.

## 5. Trace a job and control its clock

```sh
brimp run - --clock logical --epoch-ms 1700000000000 \
  --advance-time 50ms --trace trace.jsonl --events events.jsonl \
  --eval 'times' <<'JS'
globalThis.times = [];
setTimeout(() => times.push(Date.now()), 50);
document.body.textContent = 'Clock example';
null
JS
```

Expect `[1700000000050]`. Inspect `trace.jsonl` for API diagnostics and
`events.jsonl` for page/network events. Use `--trace-include PREFIX` to filter
APIs and `--trace-stack` for stacks. Logs can include headers or request bodies.

Logical time controls realm clocks and timers. Network I/O, timeout and
cancellation deadlines still use real time. Pair logical time with routes when
you need controlled network responses. `--max-tasks` bounds advancement; awaited
timer Promises may auto-advance logical time.

## Understand execution order

1. Install plugins and register init scripts, routes and event observers.
2. Navigate when a URL is supplied; init scripts precede document scripts.
3. Apply `--wait` and `--wait-selector`.
4. Run repeatable `--script` preparation files.
5. Run the main `run` script, then the action sequence.
6. Advance time, then produce the final evaluation or extracted output.

`--wait load` is the default and tracks document scripts, not image decoding.
`domcontentloaded` can return while async scripts remain outstanding.
`networkidle` uses a 500 ms quiet interval by default. A numeric wait is seconds.
The shared command timeout defaults to 30 seconds; Ctrl-C cancels active work.
Blocking OS file/terminal I/O is not a hard-real-time operation.

## Output and failures

- `html` serializes the live DOM; `raw` preserves response bytes.
- `markdown` and `json` install Defuddle and extract an article. Use `raw` for an HTTP JSON body.
- `.html`, `.htm`, `.md`, `.markdown` and `.json` infer the corresponding format.
- `--eval` / `--eval-file` await a JSON-compatible value; undefined becomes JSON null.
- Evaluation and `--format` are mutually exclusive. `--output -` needs a format unless evaluating.
- Existing result/log files need `--overwrite`. Result files publish atomically; logs may retain partial records.
- `--content`, `--remove-images`, `--language` and `--extract-debug` configure extraction output.

| Exit code | Meaning |
| ---: | --- |
| 0 | Success |
| 2 | Invalid arguments/input |
| 10 | Transport failure |
| 11 | HTTP error when `--fail-on-http-error` is set |
| 12 | Navigation failure |
| 13 | JavaScript failure |
| 14 | Timeout |
| 15 | Cancellation |
| 16 | Unsupported operation |
| 17 | Closed object or stale handle |
| 18 | Extraction, host reentry or internal runtime failure |

The [generated CLI reference](/api/cli/) lists all flags and defaults.
