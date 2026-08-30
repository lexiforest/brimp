# CLI support matrix

| Command | Tested behavior |
| --- | --- |
| `doctor` | Validates JavaScriptCore, libcurl-impersonate, and the selected curl profile. |
| `cdp` | Serves the supported Chrome DevTools Protocol subset over HTTP and WebSocket. |
| `get URL` | Writes the post-JavaScript serialized DOM to stdout. |
| `get URL --format raw|html|markdown|json|png` | Uses one navigation pipeline for response bytes, rendered DOM, live-DOM Defuddle extraction, and screenshots. |
| `get URL --eval SOURCE` | Prints one structured JSON evaluation result on stdout. |
| `get URL --script PATH` | Runs repeatable preparation scripts before capturing the selected result. |
| `crawl URL` | Runs a bounded deterministic breadth-first crawl with per-worker pages, robots policy, same-origin scope, safe atomic outputs, and a JSONL manifest. |

All commands support stable categorized exit codes. `get` has one overall
timeout, cancellation-aware fixed/selector/network-idle waits, proxy and request
state options, atomic output, and overwrite protection. `crawl` shares these
controls and adds explicit depth, worker, page, origin, path, pacing, and failure
bounds.
