# Explicit JavaScript plugins

No plugins are enabled by default. Python accepts `Plugin(name, source,
handlers={...})` in `Session(plugins=[...])`. Node accepts `new Plugin(name, source,
handlers)` or equivalent descriptors in `createSession({plugins: [...]})`.
Descriptors are fixed for the session. Installers run in order after core setup
and before document scripts, on each new realm. Source evaluates to a function
receiving `{global, host, options}`. It may return a Promise, then a disposer function or
`{dispose()}`. Navigation/closure cancel old calls, clear subscriptions, and run
disposers in reverse order. Installation errors name the plugin and original error.

`host.call(name, ...args)` always returns a Promise. Only registered handlers are
callable. Values include undefined/null, booleans, numbers (including NaN/infinities), BigInt, strings, arrays, plain objects,
and copied binary buffers. Python receives bytes; Node receives Buffer; realm
JavaScript receives Uint8Array. Use `host.ref(value)` for live objects/functions;
the handler receives a realm-bound JSHandle. `context.function(name)` returns a
registered handler as a Promise-returning page function. Raw JSC pointers never
leave the owner thread. See [value bridging](BRIDGING.md) for conversions and ownership.

Failed host calls reject in the page with
`DOMException("The operation failed.", "OperationError")`. Original adapter
errors, including Node stacks and Python tracebacks, go to the embedding
application's `page.on("error", ...)` channel with the plugin/handler name.
They are not sent to page console/error listeners. Subscribe before invoking
an API when host diagnostics are needed. Installation failures still return
the original host detail to the caller opening the page. For intentional public
failure data, return a structured result and translate it in the realm API.

Handlers receive `(context, ...args)`. Context exposes session/page/realm IDs, trusted URL/origin,
`cancelled`, and `emit(channel, value)`. JavaScript calls
`host.subscribe(channel, listener)` to receive ordered notifications and gets an
unsubscribe function. Notifications are bounded to 256 queued messages; pending
calls to 1024 per realm. Handle backpressure errors in the producer. Late results
from cancelled realms are discarded. Python uses one managed dispatcher per
session; Node uses its event loop and accepts Promise results. Callback reentry
into the same page rejects immediately, except for queued handle operations.
Those operations cannot call a host handler recursively.

```python
from brimp import Session, Plugin
plugin = Plugin("sum", "({global,host}) => { global.add = (...xs) => host.call('sum', ...xs); }",
                handlers={"sum": lambda context, *xs: sum(xs)})
with Session(plugins=[plugin]) as session:
    assert session.new_page().evaluate("add(20, 22)") == 42
```

## Bundled behavior

| Plugin | Functional | Mocked or unavailable |
| --- | --- | --- |
| Python `plugins.storage()` / Node `storagePlugin()` | Realm-local in-memory Storage, library-backed IndexedDB, Cache APIs | Fresh on navigation; no cross-page storage events, persistence, OPFS or filesystem durability |
| `plugins.host_storage(...)` / `hostStoragePlugin(...)` | Async hostStorage, origin-partitioned atomic batches, session-memory default or custom backend | Explicit extension; does not implement localStorage or IndexedDB; durability depends on the backend |
| `plugins.canvas(...)` / `canvasPlugin(...)` | Constructors, state, transforms, validation, gradients/paths, ImageData, configured dimensions | Draws do not rasterize; hit tests false; fixed-color readback; deterministic text metrics; no fonts or image decoding |
| `plugins.services(...)` / `servicesPlugin(...)` | Realm-local LockManager, MediaEncryptedEvent payloads and API validation | Device/media/permission/navigator values are configured mocks; no actual hardware, notification, geolocation, media services or encrypted-media backend |
| `plugins.extraction()` / `extractionPlugin()` | Defuddle extraction via `page.extract()` on the current DOM | Uses DOM/attributes/inline style heuristics; no fabricated layout or computed style |

Storage maps belong to the realm. This is a deliberately limited memory-only
plugin, not a persistent browser profile. IndexedDB uses pinned `fake-indexeddb`, including transactional rollback and
structured-cloned records. Its request/database fields are populated on real
instances; a prototype-only surface probe cannot verify all of them. CacheStorage
records empty caches immediately on open and preserves response body bytes.
CookieStore is a separate legacy placeholder: get/getAll/set/delete return
undefined without reading or changing document.cookie; onchange supports manual
events. SQLTransaction is an illegal-constructor interface with a legacy synthetic
success payload, and openDatabase is undefined as in the saved Safari reference.
No WebSQL engine or transaction factory is added.
StorageManager reports cache-map usage and no
quota, persistence methods resolve false, and getDirectory is a legacy no-op
returning undefined without creating a filesystem. Storage remains fresh after
navigation. Services defaults deny notifications, report unavailable Bluetooth and
empty media devices; options configure responses, not access to real devices.

Canvas drawing and path calls, including drawFocusIfNeeded, support ignored-result
workflows without rasterization. State, transforms and ImageData allocation remain
functional; readback, text metrics and hit tests use the documented mock contract.
The existing filter property remains an opt-in plugin state extension even though
it is absent from the saved Safari reference; it does not render filters.

Canvas `rgba` configures in-bounds pixels (default transparent); out-of-bounds
pixels remain transparent, and alpha-disabled contexts return opaque pixels.
`characterWidth` (Python `character_width`) controls glyph advance relative to
font size. `toDataURL`/`toBlob` produce **application/x-brimp-canvas-mock** containing
JSON dimensions/color, never a PNG. Zero-size canvases return `data:,`.
Readback is capped at 16 million pixels. Bitmap decoding rejects; natural image
sizes are zero. Canvas preserves the core image placeholders, including the
undefined-returning decode() no-op. It does not add document screenshots or layout.

The [graphics/audio example](../examples/plugins/graphics-audio.js) illustrates
JavaScript state plus explicit async backend operations. It does not implement
browser-complete WebGL/WebGPU/WebAudio. The [Node host worker example](../examples/plugins/node-host-worker.mjs)
uses worker_threads and subscriptions through an explicit `HostWorker` extension.
The services plugin exposes a Worker placeholder with local handlers and no-op
postMessage/terminate. It does not execute source, fetch a URL or start a thread.
SharedWorker, service-worker execution, and native coordinators are absent.
The services plugin supplies a ServiceWorkerContainer placeholder, not worker execution.

The services plugin also exposes Clipboard, CredentialsContainer, Geolocation,
MediaCapabilities, MediaSession and WakeLock. Their heavy operations retain legacy
no-ops returning undefined: no promises/results, callbacks, permission prompts,
clipboard access, authentication, location lookup or device work. Sharing, gamepad
and key-system navigator methods are likewise no-ops. MediaSession stores metadata
and playbackState locally. ServiceWorkerContainer.controller stays null and ready
is a stable, permanently pending promise; register does not load a worker.
MediaDevices retains empty enumeration/constraint results and adds no-op capture
methods. Permissions retains its configured synthetic query results. EventTarget
services accept manually dispatched events, without automatic device events.
PDF plugin/MIME collections retain configured metadata and indexed/named lookup;
they do not install a PDF viewer. Existing configurable connection, deviceMemory
and bluetooth surfaces remain services-plugin extensions absent in saved Safari.
TrustedTypePolicyFactory/trustedTypes in services copy the legacy placeholders:
policy creation, sink queries and type checks return undefined, defaultPolicy is
null, and emptyHTML/emptyScript are empty strings. No policies, branded trusted
values or CSP enforcement are provided. The Safari-absent getTypeMapping stays absent.
The services plugin also exposes MediaStream with a stable unique id, inactive
state and manually dispatched track handlers. Its track queries, clone and
mutation methods retain legacy undefined-returning placeholders; there are no
tracks or capture. SourceBuffer is an illegal-constructor interface only: its
methods are legacy no-ops and its property placeholders use empty strings/local
string assignments. There is no MediaSource factory, buffering or decoder.

## Additional browser surfaces

These installers are also explicit; none is enabled by default. They can be
combined with storage, services and extraction without adding native engines.

| Python / Node installer | Functional behavior | Limits and deterministic behavior |
| --- | --- | --- |
| `geometry()` / `geometryPlugin()` | DOMPoint/ReadOnly, DOMRect/ReadOnly, DOMQuad; 2D DOMMatrix/ReadOnly multiplication, inverse, translate/scale/rotate, typed arrays | No DOM measurement; non-affine/3D matrices and CSS transform-list parsing reject |
| `css_typed_om()` / `cssTypedOmPlugin()` | CSSUnitValue conversions/arithmetic, keywords, unparsed values/variable references, inline attributeStyleMap reads/edits | Single inline values; no general calc/compound-unit algebra, cascade, computed style, or layout |
| `scheduling()` / `schedulingPlugin()` | scheduler.postTask/yield, delayed priority queues, TaskController priority changes and aborts | Priority applies within this plugin's ready tasks; idle callbacks report zero available budget; no rendering-idle prediction or inherited continuation priorities |
| `graphics(...)` / `graphicsPlugin(...)` | Canvas WebGL/WebGL2 context identity, buffer/state bookkeeping, Safari constants and legacy callable placeholders; opt-in WebGPU buffer allocation, mapping and copying | Fixed RGBA readback; shaders report compilation/linking failure; additional GL methods return undefined without effects; WebGPU execution rejects; no rasterization or native GPU |
| `audio(...)` / `audioPlugin(...)` | AudioBuffer channel data, graph connections, listener/compressor and Gain/Oscillator/Analyser state, AudioContext lifecycle and realm-clock time | Silent analysis; missing DSP factories, automation ramps and worklet loading are callable no-ops; decoding rejects; offline rendering returns a silent buffer, with no processing/output |
| `rtc()` / `rtcPlugin()` | Local peer/data-channel objects, description and candidate payload storage, RTC event payloads/handlers | No connection or media transport; legacy no-op operations and a permanently pending createOffer promise |

The RTC plugin is opt-in in Rust (`bundled::rtc()`), Node and Python. Peer and
channel defaults use the saved Safari reference; sending, closing, negotiation,
configuration and receiver/statistics calls remain legacy no-ops returning
undefined. close does not transition state. createOffer never settles and does
not invoke callbacks. setLocalDescription only stores the supplied value as local
and pendingLocalDescription and resolves; it does not parse SDP or negotiate.
Session descriptions and candidate input fields can be serialized with toJSON.
Candidate-derived network fields are null placeholders, without parsing or a
fabricated local address. RTCSctpTransport is an illegal-constructor surface with
legacy empty-string getters; peers never create a transport. The plugin produces
no network/device events; manually dispatched events use the normal EventTarget.
Legacy members absent in the saved Safari reference are omitted.

The graphics plugin accepts `rgba`, `renderer`, and `webgpu` (default false).

The audio plugin includes BaseAudioContext, AudioScheduledSourceNode,
AudioDestinationNode, AudioListener, DynamicsCompressorNode, Worklet/AudioWorklet
and OfflineAudioContext. Missing DSP factories return undefined; they do not
construct processing nodes. Worklet.addModule returns undefined and loads no code.
OfflineAudioContext.startRendering resolves to an AudioBuffer filled with zeros;
it does not render a graph or emit completion events. Its legacy suspend/resume
placeholders return promises that remain pending. Existing buffer manipulation,
graph connections, logical AudioContext time, and host audio.process extensions
remain available. The legacy numberOfChannels property stays absent on the
offline context itself, matching Safari; the buffer exposes its channel count.
SpeechSynthesisUtterance stores local text/language, numeric pitch/rate/volume and
voice payloads with Safari's empty-utterance defaults. Its event handlers support
manual dispatch only. Numeric assignments are stored without full Web IDL/range
validation. speechSynthesis copies the legacy empty object: no speech methods,
queue, installed voices or audio output are provided.
The canvas plugin adds legacy captureStream/transferControlToOffscreen no-ops.
Path2D calls retain validation and opaque path identities without rendering.
Existing toBlob/toDataURL encode mock metadata, and createImageBitmap supplies
mock dimensions; these do not imply image codecs or rasterization.

The graphics plugin exposes WEBGL_debug_renderer_info with a stable extension
object, Safari's two constants and the existing configured renderer/Brimp vendor
strings. It reports mock identity, not hardware. Its other WebGL constants use
Safari values and readonly descriptors. The additional
109 methods listed in `placeholderMethods` in `crates/runtime/js/plugins/graphics.js`
copy the legacy callable-placeholder contract: they return `undefined` without
converting arguments, changing GL state or executing GPU work. This includes
texture/framebuffer creation and queries; scripts can discard these results,
but cannot use them as real resources or query answers. Method names, arities
and descriptors match installed Safari. Existing buffer, viewport, clear-color
and fixed-pixel readback behavior remains available. These are explicit plugin
contracts, not full WebGL semantics; `COVERAGE.yaml` records them individually.

Without `webgpu: true`, `navigator.gpu.requestAdapter()` returns null. Mock GPU
buffers are capped at 64 MiB and support full-buffer mappings only; old mapping
views stop affecting buffer contents after unmap. These are memory mocks, not a
WebGPU conformance implementation. When using the Canvas plugin too, install
Canvas **before** graphics so the graphics installer composes its getContext
behavior. Audio sample rate defaults to 48000 (`sample_rate` in Python,
`sampleRate` in Node); buffers are capped at 16 million samples.

Graphics exposes explicit `submitAsync` extensions invoking `graphics.submit`.
AudioContext exposes `processAsync`, invoking `audio.process`. Supply these
handlers through the installer; synchronous mock methods never wait for host
work. Host calls use the existing copied-binary/cancellation/reentry contract.

```python
plugin = plugins.audio(handlers={
    "audio.process": lambda context, request: request["samples"]
})
```

```javascript
const plugin = graphicsPlugin({webgpu: true}, {
  'graphics.submit': async (context, commands) => userBackend.submit(commands)
});
```

Geometry and Typed OM subsets follow the data models in the
[Geometry Interfaces specification](https://www.w3.org/TR/geometry-1/) and
[CSS Typed OM specification](https://www.w3.org/TR/css-typed-om-1/). Scheduling is
based on the [Prioritized Task Scheduling draft](https://wicg.github.io/scheduling-apis/).
The subset limits above are part of the contract, not claims of full standards
coverage. Crypto is [core](CORE_WEB.md), with an explicit supported algorithm
set. Worker/SharedWorker/service-worker execution, full SVG/CSSOM, media codecs
and WebRTC remain outside core; use explicit host plugins where needed.

## File loading, options and state lifetime

Node: `Plugin.fromFile(pathOrURL, {name, options, handlers, setup})`.
Python: `Plugin.from_file(path, name=..., options=..., handlers=..., setup=...)`.
The file must evaluate to an installer; bundle module dependencies into that
installer first (for example with esbuild). Original file URLs are retained in
host-side installation diagnostics; evaluated installers use a neutral `web-api`
source label (the engine may omit it from stacks). Options
are serialized into each realm. [realm-library.js](../examples/plugins/realm-library.js)
shows a small realm-only installer with options and cleanup.

Core and plugin APIs are finalized together before page scripts: directly
installed functions, namespace objects, prototypes and accessors receive the
runtime's built-in source representation without executing getters. Property
flags and object identity are preserved by source registration; existing core
Web IDL receiver checks still apply. Application scripts remain free to replace
methods or add globals. This is initialization-time API finalization, not a
continuous rewrite of application objects or dynamically generated functions.

Plugin installers are trusted code and must keep their own state in closures,
private fields or WeakMaps and expose only intended APIs. Deliberately exported
options, secrets, mock behavior or nonstandard APIs remain observable. The core
boundary is not a sandbox for a malicious plugin. See [CORE_WEB.md](CORE_WEB.md)
for the page exposure contract.

`setup` is a synchronous host-side factory, invoked once per Session. It returns
`{handlers, dispose}` (a dict in Python). Factory handlers extend the configured
handlers. Its state survives page navigation and is independent for each session,
even if the same Plugin object is reused. Close the Session to release it. Session
cleanup runs once, in reverse order, after native pages close; failed session setup
also disposes scopes already created. Node disposers may be async; Python factories,
handlers and disposers are synchronous. Realm installer disposers still run on each
navigation/closure. A failed page installer does not close its owning session.

Use `context.origin` for origin partitions, and reject `null`/`None` when an API
requires a non-opaque origin. This value and `context.url` come from the native
browsing context, not page JavaScript. `sessionId`/`session_id` identifies the
session, `pageId`/`page_id` survives navigation, and `realmId`/`realm_id` changes.
Pending invocations retain their captured context. Disposal of an old realm runs
before the next document's URL becomes current. Host state shared across pages
must be concurrency-safe in Rust; Node/Python adapters dispatch on their managed
host loop/thread.

The [Node](../examples/plugins/session-preferences.mjs) and
[Python](../examples/plugins/session_preferences.py) examples implement an async,
origin-partitioned preferences API that survives reload within a session. They
illustrate host lifetime without claiming browser IndexedDB compatibility. Do not
hide an async `host.call()` behind a synchronous browser API. A persistent IndexedDB
backend needs transaction ownership and cross-page coordination, not a realm-exit
snapshot of an in-memory database.

## Discovering capabilities and choosing storage lifetime

Node `pluginCatalog()` and Python `plugins.catalog()` return an independent
host-side catalog of installer names, capabilities, limitations, option defaults
and lifetimes. Discovery installs nothing in a page. Options marked hostOnly
(such as a backend factory) stay in the embedding application.

`hostStoragePlugin()` / `plugins.host_storage()` explicitly install the async
`hostStorage` extension. Its default memory backend is owned by the Session,
partitioned by the native origin, shared by that session's same-origin pages,
and survives navigation. Closing the Session discards it. Opaque origins reject.
It exposes get/set/delete/clear and transaction(operations), where a batch contains
`{type:'get',key}`, `{type:'set',key,value}`, `{type:'delete',key}` or `{type:'clear'}`.
A transaction returns results in order; set/clear return null and delete returns
a boolean. The default backend validates the whole batch and commits atomically.

```javascript
const session = await createSession({plugins: [hostStoragePlugin()]});
// On an HTTPS page: await page.evaluate('hostStorage.set("theme", "dark")');
```

For a custom backend, pass Node `backendFactory` or Python `backend_factory`.
The synchronous factory runs once per Session and returns an object implementing
`transaction(origin, operations)` and optional `close()`. Node transactions/close
may be async; Python uses synchronous methods. Node serializes batches per origin,
and Python serializes backend access; cleanup waits for in-flight backend access.
The backend itself must guarantee atomic commit/rollback and coordinate other
sessions/processes if it shares durable state. Cancellation before a batch starts
prevents execution; cancellation during a committed backend transaction cannot
undo its writes. Late results are still discarded by the realm bridge.

[SQLiteStorage](../examples/plugins/sqlite_storage.py) is a runnable Python backend
example using explicit BEGIN IMMEDIATE/COMMIT/ROLLBACK, native-origin partitions
and cross-connection coordination. Supply its path explicitly to opt into disk
persistence. It accepts JSON values; the default memory backend also accepts
copied binary values. It has tests for rollback, independent origins, visibility
across connections and reopen.

This extension is not localStorage, sessionStorage or IndexedDB. The existing
`storagePlugin()` / `plugins.storage()` remains realm-local and loses data on
navigation. A durable browser IndexedDB backend remains separate work requiring
transaction scheduling, versionchange/blocked behavior and cross-page events.
