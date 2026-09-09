# brimp-controller

Rust controller library and `brimp` CLI for separate browser worker processes.
It provides browser operations and lifecycle management, HTTP discovery, CDP
WebSockets, worker pooling, crash recovery, and resource limits. The controller
does not link WebKit, JavaScriptCore, curl-impersonate, or the lite runtime.

The controller implements the stable Chrome DevTools Protocol **1.3** shape,
pinned to the `devtools-protocol` package version **0.0.1682007**. The pin is a
wire-format reference, not a claim that every command in that schema is
implemented. Unsupported commands always return JSON-RPC error `-32601`.

`WebKitAutomationWorker` speaks CDP JSON directly but does not host HTTP or a
WebSocket server. Brimp gives each worker one end of a private local socket
using `--controller-socket-fd`. Each UTF-8 CDP message is prefixed by a
four-byte network-byte-order payload length:

```json
{"id":1,"method":"Page.navigate","sessionId":"abc","params":{"url":"https://example.test/"}}
{"id":1,"sessionId":"abc","result":{"frameId":"main","navigationId":"0x1234"}}
{"method":"Page.loadEventFired","sessionId":"abc","params":{"timestamp":1.25,"navigationId":"0x1234"}}
```

The first command is `Browser.getVersion`; a compatible worker reports CDP
protocol version `1.3`. A malformed frame, EOF, timeout, or mismatched version
causes the controller to kill and replace the entire worker process group.
Frames are limited to 64 MiB so encoded screenshots remain bounded. Standard
output and standard error are reserved for diagnostics and cannot corrupt IPC.

## Managed browser connections

All controller browser sessions own their worker process. Direct commands use
`--worker-path PATH`, optional `--cdp`, `--worker-args PATH`, and repeatable
`--worker-arg ARG`. Argument files contain a JSON array of strings, passed before
inline arguments. With `--cdp`, the controller launches a headless child with a
private temporary profile and loopback port, discovers `/json/version`, and
connects to its browser WebSocket automatically. Existing WebSocket servers
are not accepted as browser inputs.

The browser API and server pool share startup and discovery in `worker.rs`.
Startup, discovery, and handshaking count toward the operation deadline.
Cancellation, failure, and completion terminate and reap the owned process group;
normal closure also disposes the session's browser context. Temporary profiles
are removed after process cleanup. The framed transport also has a Windows
named-pipe adapter, but Windows worker launch is not implemented.

## Managed CDP proxy workers

With `--cdp`, the child process already speaks CDP and the
private framed-CDP protocol is not used. The Rust proxy assigns every
child a unique loopback port, waits for `/json/version`, and exposes rewritten
discovery URLs from the controller listener. Worker arguments may contain
`{port}`; otherwise the controller appends `--remote-debugging-port=PORT`.

Affinity is lease based. A discovery URL contains a cryptographically random
token, and its first WebSocket upgrade atomically assigns one available worker.
All browser and direct-target WebSockets carrying that token connect to the
same process. CDP frames are forwarded byte for byte, so backend `sessionId`,
target, context, and remote-object semantics remain owned by that process.

Workers are exclusive to a lease. When its final WebSocket closes, the
controller terminates and replaces the process instead of attempting to reset
opaque browser state. A process exit or memory-limit violation invalidates the
lease and starts a replacement; connections are never silently moved because
their CDP identifiers would be invalid. Unclaimed discovery tokens expire after
ten seconds, and `--max-pending` bounds live tokens when it is nonzero.

## Version-one command matrix

| Domain | Commands |
|---|---|
| Browser | `getVersion`, `close` |
| Target | `getTargets`, `setDiscoverTargets`, `setAutoAttach`, `createTarget`, `closeTarget`, `attachToTarget`, `detachFromTarget`, `getBrowserContexts`, `createBrowserContext`, `disposeBrowserContext` |
| Page | `enable`, `disable`, `setLifecycleEventsEnabled`, `navigate`, `reload`, `getFrameTree`, `captureScreenshot` |
| Runtime | `enable`, `disable`, `evaluate`, `callFunctionOn`, `releaseObject` |
| Network | `enable`, `disable` |
| Audits, Performance, Log | `enable` (event subscriptions; no events or metrics are currently emitted) |
| Emulation | `setDeviceMetricsOverride` (viewport size), `setTouchEmulationEnabled` (accepted; touch emulation is unavailable) |
| Input | `dispatchMouseEvent`, `dispatchKeyEvent` |

Target sessions are flattened. `Page.navigate` acknowledges the native load
request and lifecycle events arrive separately. The private `navigationId`
correlates those events to the returned `WKNavigation`; the controller drops
stale events and removes this private field before emitting CDP. Browser
contexts are always ephemeral. Each context leases one worker and owns one
nonpersistent `WKWebsiteDataStore`; all targets in that context share its
cookies and origin storage, while targets in other contexts remain isolated.
Disposing the context closes all of its targets, releases the data store, and
returns the worker to the pool. The former `verificationIdentity` persistence
extension is rejected so callers cannot accidentally assume data will survive.

`Network.getResponseBody` is deliberately unsupported: the macOS
`_WKResourceLoadDelegate` reports request, response, completion, and failure
metadata but does not expose response bytes. The controller therefore returns
`-32601` instead of fabricating a body. Full-page screenshots are assembled
from native viewport snapshots while scrolling the offscreen `WKWebView`; the
worker restores the original scroll position afterward and rejects documents
larger than its bounded bitmap limit.

The controller emits `Target.targetCreated`, `Target.targetInfoChanged`,
`Target.targetDestroyed`, `Target.attachedToTarget`, and
`Target.detachedFromTarget`. Worker events already use CDP method names; Brimp
routes them to the correct external session. Lifecycle and execution-context
events marked below are derived by Brimp:

| Worker CDP event | External CDP event |
|---|---|
| `Page.frameStartedLoading` | `Page.frameStartedLoading` |
| `Page.frameNavigated` | `Page.frameNavigated` |
| `Page.frameNavigated` | derived `Page.lifecycleEvent` named `init` with the navigation loader ID |
| `Page.domContentEventFired` | `Page.domContentEventFired` plus derived `Page.lifecycleEvent` named `DOMContentLoaded` |
| `Page.loadEventFired` | `Page.loadEventFired` plus derived `Page.lifecycleEvent` named `load` |
| `Page.frameStoppedLoading` | `Page.frameStoppedLoading` |
| `Network.requestWillBeSent` | `Network.requestWillBeSent` |
| `Network.responseReceived` | `Network.responseReceived` |
| `Network.loadingFinished` | `Network.loadingFinished` |
| `Network.loadingFailed` | `Network.loadingFailed` |

`Runtime.executionContextCreated` is synthesized when an external session
enables the Runtime domain because Cocoa WebKit does not expose CDP execution
contexts directly.

## Worker modes

`--headless --window-size=1280,720` keeps a normal `WKWebView` attached to a
borderless offscreen `NSWindow` under the accessory application policy. It does
not disable GPU compositing or override JavaScript-visible headless signals.
The host disables WebKit window-occlusion detection because a fully offscreen
window otherwise stops animation frames; this is an embedding activity-state
choice, not a JavaScript shim.

The raw acceptance test verifies a configured 640x480 viewport, native
focus/input, overflow capture, and screenshots before and after scrolling in
headless mode. Historical MiniBrowser headed/headless measurements remain in
the benchmark reports, but should not be treated as measurements of this new
dedicated worker executable.

Input support currently covers mouse move/press/release/wheel and key
down/up/character text. Touch, IME composition, and drag semantics are not in
the version-one matrix and fail explicitly.


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
adjacent WebKit checkout. For an `.app` path, the controller sets the framework
and library search paths for the worker and its XPC services to the app's parent
directory, where WebKit's build places its matching frameworks. When passing a
bare executable, supply the framework environment to the CLI yourself.
For Chrome, supply the actual browser executable:

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
