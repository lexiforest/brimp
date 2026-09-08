# Brimp CDP contract

The controller implementation is Rust. Platform adapters provide only
process-tree management, connected local streams, loopback sockets, secure
randomness, CPU discovery, and memory accounting. Both the external CDP wire
contract and private worker protocol are shared across ports.

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

## Managed CDP proxy workers

With `--worker-protocol=cdp`, the child process already speaks CDP and the
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

## Puppeteer compatibility smoke test

The compatibility target is pinned to `puppeteer-core` **25.10.0**. Its page
bootstrap also enables Audits, Performance, and Log; requests viewport metrics;
and creates Puppeteer's named utility world. The controller accepts the exact
bootstrap pattern, publishes stable synthetic execution-context IDs, and runs
evaluation through WebKit's page world. General preload-script and isolated-
world semantics are not part of version one.

With a controller already listening on port 9222, the pinned smoke test can be
run without downloading Chromium:

```sh
npm install --prefix /tmp/minibrowser-puppeteer puppeteer-core@25.10.0
NODE_PATH=/tmp/minibrowser-puppeteer/node_modules \
    node tests/puppeteer_smoke_test.js http://127.0.0.1:9222
```

It covers `connect()`, existing-page discovery, HTTP navigation with load
readiness, `page.evaluate()`, and `page.screenshot()`.

## Starting a macOS Release controller

Build `WebKitAutomationWorker` from the adjacent WebKit checkout first. Brimp
does not link WebKit; `--worker-path` selects the matching worker application
or executable. The controller starts two workers per physical CPU core unless
`--pool-size` is supplied.

```sh
DYLD_FRAMEWORK_PATH=$PWD/../WebKit/WebKitBuild/Release \
__XPC_DYLD_FRAMEWORK_PATH=$PWD/../WebKit/WebKitBuild/Release \
./target/release/brimp --port 9222 \
    --worker-path=$PWD/../WebKit/WebKitBuild/Release/WebKitAutomationWorker.app \
    --headless --window-size=1280,720
```

Run `python3 tests/cdp_smoke_test.py http://127.0.0.1:9222` for the raw
acceptance suite.
