# CLI support matrix

| Command | Tested behavior |
| --- | --- |
| `doctor` | Validates JavaScriptCore, libcurl-impersonate, and the selected curl profile. |
| `get URL` | Writes the post-JavaScript serialized DOM to stdout. |
| `get URL --format raw|html|markdown|json|png` | Uses one navigation pipeline for response bytes, rendered DOM, live-DOM Defuddle extraction, and screenshots. |
| `get URL --eval SOURCE` | Prints one structured JSON evaluation result on stdout. |
| `get URL --script PATH` | Runs repeatable preparation scripts before capturing the selected result. |
| `get URL --cookie NAME=VALUE` | Seeds the browser-context cookie jar once before navigation; normal cookie scoping applies. |
| `crawl URL` | Runs a bounded deterministic breadth-first crawl with per-worker pages, robots policy, same-origin scope, safe atomic outputs, and a JSONL manifest. |

All commands support stable categorized exit codes. `get` has one overall
timeout, cancellation-aware fixed/selector/network-idle waits, proxy and request
state options, atomic output, and overwrite protection. `crawl` shares these
controls and adds explicit depth, worker, page, origin, path, pacing, and failure
bounds.

## Worker boundary

All browser commands require `--worker-path` or `BRIMP_WORKER_PATH` and currently
run only on macOS. `get`, `crawl`, and `doctor` launch a framed-CDP child;
`serve` owns the public HTTP/WebSocket endpoint and worker pool. The CLI binary
has no native browser dependency. Python and Node bindings remain in-process.

Lite workers retain the command features above. The current WebKit worker
supports rendered output, evaluation, extraction, and screenshots; raw bodies,
robots retrieval, headers/cookies, and lite-specific configuration are rejected.
Worker death produces a failure without automatically replaying the workload.
