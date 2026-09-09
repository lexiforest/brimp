# brimp-controller

Rust controller library and `brimp` CLI for separate browser worker processes.
It provides browser operations and lifecycle management, HTTP discovery, CDP
WebSockets, worker pooling, crash recovery, and resource limits. The controller
does not link WebKit, JavaScriptCore, curl-impersonate, or the lite runtime.

## Build and install

From the workspace root:

```sh
cargo build -p brimp-controller -p brimp-lite-worker
```

The CLI alone builds without native browser SDKs:

```sh
cargo build -p brimp-controller
```

Tagged releases publish relocatable archives for manylinux 2.28 x86-64/ARM64,
macOS 11+ ARM64, and Windows x86-64. Each archive includes the required native
runtimes, their licenses, and a separate SHA-256 checksum.

## Managed browser workers

`fetch`, `crawl`, `doctor`, and `serve` launch and manage their workers. Select
an executable with `--worker-path PATH` or `BRIMP_WORKER_PATH`. By default the
controller launches a managed framed worker; add `--cdp` to launch a WebSocket
CDP worker instead. Worker launch and supervision currently require macOS.

| Selection | Worker connection |
| --- | --- |
| Default | Pass an inherited socketpair to a lite or WebKit worker. |
| `--cdp` | Launch a WebSocket CDP worker such as Chrome, and connect automatically. |

With `--cdp`, the controller launches a new browser with profile isolated from existing
Chrome sessions and is removed after process cleanup.

Repeat `--worker-arg ARG` to pass worker arguments. `{port}` placeholders are
replaced with the controller's allocated port; otherwise it appends
`--remote-debugging-port=PORT`. The controller owns the debugging endpoint and
user-data directory. Existing browser endpoints and profiles are not inputs.

`--worker-args PATH` reads launch arguments from a UTF-8 JSON array of strings.
Each string is one argument, preserving spaces and quotes without shell expansion.
File arguments are passed first, followed by repeatable `--worker-arg` values.
The option works for `fetch`, `crawl`, `doctor`, and `serve`, with either worker mode.
For example, `chrome-args.json` can contain:

```json
["--disable-gpu", "--window-size=1280,720"]
```

```sh
brimp fetch https://example.com --worker-path /path/to/chrome --cdp --worker-args chrome-args.json
```

A framed worker path accepts a lite-worker executable or a matching
`WebKitAutomationWorker.app`/executable. WebKit is built separately from the
adjacent WebKit checkout; supply any required framework environment to the CLI
so its children inherit it. For Chrome, supply the actual browser executable:

```sh
export BRIMP_WORKER_PATH="$PWD/target/debug/lite-worker"
brimp doctor

brimp fetch https://example.com \
  --worker-path '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
  --cdp --eval document.title
```

## CLI usage

With a worker selected through `BRIMP_WORKER_PATH`:

```sh
brimp fetch https://example.com
brimp fetch https://example.com --output article.md
brimp fetch https://example.com --format json --output article.json
brimp fetch https://example.com --eval 'document.title'
brimp fetch https://example.com --output page.png --full-page
brimp crawl https://example.com --output-dir ./reference --depth 2 --concurrency 2
```

Results go to stdout or `--output`, and diagnostics stay on stderr. Rendered
HTML, raw response bytes, Defuddle Markdown/JSON, evaluation JSON, and PNG are
supported. Existing files are refused unless `--overwrite` is explicit.

`fetch` creates one page. `crawl` uses `--concurrency` concurrent pages sharing a
browser context. It writes Markdown by default and records one terminal JSON
object per URL in `manifest.jsonl`. It obeys robots policy and remains on the
final start origin unless explicitly expanded with `--allow-origin`.

Scheduling, robots policy, and output files belong to the CLI; network and page
operations run in the selected browser. Startup counts toward the command
deadline, including discovery and the WebSocket handshake. Completion, timeout,
cancellation, and errors clean up the owned worker process group. Requests are
not automatically replayed after a crash.

### Tested command support

| Command | Tested behavior |
| --- | --- |
| `doctor` | Reports worker identity and protocol readiness; lite workers also validate JavaScriptCore, libcurl-impersonate, and the selected curl profile. |
| `fetch URL` | Writes the post-JavaScript serialized DOM to stdout. |
| `fetch URL --format raw|html|markdown|json|png` | Uses one navigation pipeline for response bytes, rendered DOM, live-DOM Defuddle extraction, and screenshots. |
| `fetch URL --eval SOURCE` | Prints one structured JSON evaluation result on stdout. |
| `fetch URL --script PATH` | Runs repeatable preparation scripts before capturing the selected result. |
| `fetch URL --cookie NAME=VALUE` | Seeds the browser-context cookie jar once before navigation; normal cookie scoping applies. |
| `crawl URL` | Runs a bounded deterministic breadth-first crawl with concurrent pages, robots policy, same-origin scope, safe atomic outputs, and a JSONL manifest. |

All commands support stable categorized exit codes. `fetch` has one overall
timeout, cancellation-aware fixed/selector/network-idle waits, proxy and request
state options, atomic output, and overwrite protection. `crawl` shares these
controls and adds explicit depth, worker, page, origin, path, pacing, and failure
bounds.

### Worker capabilities

Lite workers support the command features above. The current WebKit worker
supports rendered HTML, extraction, evaluation, and screenshots. It rejects raw
response retrieval, robots.txt retrieval, custom headers/cookies, personas,
proxy/trust-root configuration, and lite subsystem/storage options with explicit
errors. WebKit crawling currently requires `--ignore-robots`.

### Exit codes

| Code | Category |
| --- | --- |
| 2 | Input |
| 10 | Transport |
| 11 | HTTP status |
| 12 | Navigation |
| 13 | JavaScript |
| 14 | Timeout |
| 15 | Cancellation |
| 16 | Unsupported result |
| 17 | Closed object |
| 18 | Extraction, screenshot, or runtime failure |

## CDP server

`serve` owns the public HTTP/WebSocket endpoint and worker pool. Local worker
launch and server process supervision currently require macOS.

```sh
./target/debug/brimp serve --worker-path "$PWD/target/debug/lite-worker" --pool-size=2 --port=9222
```

With `BRIMP_WORKER_PATH` set, this is also available as `brimp serve --port=9222`.
For workers that expose HTTP/CDP themselves:

```sh
brimp serve --cdp --worker-path=/path/to/browser --pool-size=2 --port=9222
```

The server uses the same managed CDP launcher as direct browser commands.
Closing a lease replaces its worker.

## Layers

```text
main.rs → cli/                         arguments, output, crawl scheduling
            ↓
         browser.rs                    operations, contexts, pages, lifecycle
            ↓
         connection.rs                 CDP requests, responses, session events
            ↓
         transport::Transport          bounded, nonblocking message channel
            ├── SocketPair             Unix length-prefixed JSON
            ├── NamedPipe              Windows length-prefixed JSON
            └── WebSocketTransport     ws:// CDP text messages
```

`server/` owns the public CDP facade, worker pools, and transparent WebSocket
proxy. Its framed workers use the same transport implementation. The transparent
proxy preserves backend messages and identifiers instead of interpreting page
operations. `mac/platform.rs` owns OS process creation and supervision.

`Browser::launch` takes `WorkerOptions` (executable, protocol, and arguments),
launches the child, connects its transport, and creates a context. It owns the
child process for the entire session. `worker.rs` shares WebSocket CDP startup
and discovery with the server pool. Closing disposes the context when possible,
closes the connection, and terminates and reaps the process group. Explicit close
and `Drop` are idempotent; surviving page handles cannot keep a worker alive.

Socketpair, named-pipe, and WebSocket modules implement message transfer. The
named-pipe adapter is available for Windows platform integration, but a Windows
worker launcher has not been implemented. There is no attach-only browser API
or CLI command.

The CDP connection handles response correlation, bounded session-event queues,
cancellation, and an overall deadline. Transports only move messages and retain
partial reads/writes; local streams use four-byte big-endian lengths. Messages
and queued writes are bounded around the protocol's 64 MiB message limit.

## Validation

```sh
cargo build -p brimp-controller -p brimp-lite-worker
cargo test --workspace
python -m pytest crates/controller/tests
# Optional full WebKit acceptance:
BRIMP_TEST_WEBKIT_WORKER="$PWD/../WebKit/WebKitBuild/Release/WebKitAutomationWorker.app" python -m pytest crates/controller/tests -k webkit
```

The imported controller and worker contract is documented in
[docs/PROTOCOL.md](docs/PROTOCOL.md). Lite CDP support is documented separately
in [../worker/SUPPORT.md](../worker/SUPPORT.md).
