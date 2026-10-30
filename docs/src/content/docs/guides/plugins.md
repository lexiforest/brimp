---
title: Plugins
description: Install browser capabilities and connect page scripts to host handlers.
---

Default sessions install environment, permissions, devices, media, workers,
trusted-types, canvas, graphics, audio and RTC plugins. Their synthetic behavior
is part of the default browser environment; it does not require real devices or
media/rendering engines. Extraction and host storage remain explicit integrations.

Supplying a `plugins` list selects the complete set, replacing the defaults.
Use `plugins: []` in Node or `plugins=[]` in Python for core-only execution.
Use `browserPlugins()` in Node, `plugins.browser()` in Python or `bundled::browser()`
in Rust to obtain fresh default descriptors before adding or replacing a plugin.
Rust `SessionOptions` and `PageOptions` default to this browser set. The CLI uses
it when no `--plugin` is supplied; repeated `--plugin` flags select a complete set,
and `--no-plugins` selects core only. Markdown/JSON output still adds extraction.
The plugin catalog reports `defaultEnabled` for each installer.

Python accepts `Plugin(name, source,
handlers={...})` in `Session(plugins=[...])`. Node accepts `new Plugin(name, source,
handlers)` or equivalent descriptors in `createSession({plugins: [...]})`.
Descriptors are fixed for the session. Installers run in order after core setup
and before document scripts, on each new realm. Source evaluates to a function
receiving `{global, host, options, profile}`. The `profile` registry lets an installer
[own its configurable values and browser capture recipe](/guides/profiles/). It may return a Promise, then a disposer function or
`{dispose()}`. Navigation/closure cancel old calls, clear subscriptions, and run
disposers in reverse order. Installation errors name the plugin and original error.

`host.call(name, ...args)` always returns a Promise. Only registered handlers are
callable. Values include undefined/null, booleans, numbers (including NaN/infinities), BigInt, strings, arrays, plain objects,
and copied binary buffers. Python receives bytes; Node receives Buffer; realm
JavaScript receives Uint8Array. Use `host.ref(value)` for live objects/functions;
the handler receives a realm-bound JSHandle. `context.function(name)` returns a
registered handler as a Promise-returning page function. Raw JSC pointers never
leave the owner thread. See [value bridging](/guides/values/) for conversions and ownership.

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

## Rust and JavaScript responsibilities

The runtime uses JavaScript for browser-facing objects and behavior, and Rust
for native resources, authoritative state and runtime control. Most web APIs
span both layers; JavaScript contains substantial implementation, not just
wrappers around Rust calls.

| Area | JavaScript responsibilities | Rust/native responsibilities |
| --- | --- | --- |
| DOM | Classes, properties, collections and browser behavior | Canonical Lexbor tree, parsing, queries and mutations |
| Events and forms | Event propagation, listeners, validation and control behavior | Underlying DOM access and host actions |
| Fetch and WebSocket | Browser objects, Promises and stream interfaces | Network I/O, cookies, cancellation and buffering |
| Timers | Public methods and callback arguments | Clock, deadlines, scheduling and callback lifetime |
| WebCrypto | Algorithm normalization, binary input conversion and Promise interfaces | Key ownership, authoritative usage/extractability checks and BoringSSL calls |

JavaScript makes prototypes, property descriptors, coercion, callbacks and object
identity straightforward to express. It also allows reuse of libraries such as
`web-streams-polyfill`. Rust connects those objects to native libraries and OS
resources, manages lifetimes, and enforces rules that changes to JavaScript
wrappers must not bypass. For example, modifying a CryptoKey wrapper cannot grant
permission to export a native non-extractable key.

When deciding where new behavior belongs:

1. Keep authoritative state with its existing owner. Avoid parallel Rust and
   JavaScript copies of the same state.
2. Put native resource access, lifetime management and trusted enforcement in Rust.
3. Prefer JavaScript for browser object behavior that can use those native operations.
4. Measure before moving code for performance. Frequent boundary crossings and
   data copies can cost more than the computation itself.

This describes the intended division of responsibilities. Existing code follows
it broadly, but some placement reflects how individual features developed.

## Core runtime and plugins

Core versus plugin is a separate decision from Rust versus JavaScript. Both
core APIs and plugins can use JavaScript implementations backed by native code
or registered host handlers.

| Core runtime | Plugin |
| --- | --- |
| Installed in every page | Installed according to the session's plugin selection |
| DOM APIs and shared runtime infrastructure | Cohesive browser subsystems and application-specific integrations |
| DOM, events, forms and shared realm/scheduling infrastructure | Audio, graphics, RTC, device services, extraction and host storage |
| Lifecycle managed directly by the runtime | Installer, optional host handlers and cleanup hooks |

Brimp runs external browser scripts, including legacy code that calls an API but
does not use its result. Keeping those scripts running is the goal; complete web
standards conformance is not. A callable placeholder, synthetic result or callback
can therefore be required runtime behavior. Incomplete implementation does not
make an API optional, and replacing a harmless call with a throw or rejection is
a compatibility change even if it follows a specification more closely.

Core owns DOM APIs and shared runtime infrastructure. Plugins own browser
subsystems such as audio, graphics, RTC and device services, including their
constructors, objects, placeholders, synthetic behavior and backend integration.
An audio plugin can provide useful browser-script compatibility without a real
audio backend. Required compatibility behavior does not by itself belong in core.

Ownership, default activation and backend capability are separate decisions.
A plugin can be included in the default browser environment; an optional backend
does not imply an optional API surface. Browser subsystem plugins are enabled by
default. Their existing configured exposure rules remain in effect, including explicit mock extensions to the browser
profile; default activation does not turn those mocks into real backends.

Browser services are split into environment, permissions, devices, media, workers
and Trusted Types plugins, preserving their APIs, mocks and configuration options.
Keep each subsystem's surface and implementation together rather than creating a separate switch for every placeholder. Preserve
useful callability, return shapes, callbacks and promise behavior when moving code;
assess intentional behavior changes separately against script needs. User plugins
can enhance or replace subsystem behavior.

The tables below describe the installed behavior and its limits. Browser subsystem
plugins are enabled by default; explicit plugin selection can replace that set.

Plugins should use the runtime's DOM, scheduling and realm lifecycle rather than
maintaining competing implementations of those shared resources.

Currently, geometry, CSS Typed OM, ImageData, navigator identity, Web Locks and
browser storage are installed in core. SpeechSynthesisUtterance belongs to audio;
MediaEncryptedEvent belongs to media. This
describes current placement, not a rule that all backend-independent objects must
be in core. Subsystem-specific objects should be reviewed with their owning
subsystem. Canvas context selection is also currently core; plugins register
the contexts they implement.

## Overriding runtime behavior

A user plugin can replace much of the JavaScript-facing API. Its installer runs
in the page's JavaScript realm after core initialization and before document
scripts, with access to the global object. It can replace writable globals such
as `fetch`, patch prototype methods, replace configurable getters, and provide
alternative implementations backed by registered host handlers. Canvas rendering backends use explicit registration; custom plugins may also
replace configurable JavaScript methods directly.

There are three limits:

- **Property descriptors:** a non-writable, non-configurable property cannot be
  replaced. Inspect its descriptor before attempting an override.
- **Captured references:** core code may retain an original function internally.
  Replacing the public property does not redirect calls through those references.
- **Native ownership:** JavaScript patches cannot replace Lexbor, the native
  scheduler, or native key-policy checks. Replacing page `fetch` does not
  automatically change host `page.get()` navigation. A replacement crypto API
  can implement its own behavior, but cannot bypass the native checks on keys
  owned by the core implementation.

Installers run in descriptor order, so a later plugin can overwrite an earlier
plugin's changes where property descriptors permit. Each new realm runs the
installers again. Overrides share the page's realm and must preserve the browser
semantics that their consumers depend on.

This mechanism patches JavaScript APIs. Replacing an underlying native subsystem
requires an explicit interface for that subsystem; plugins do not provide a
general native-backend replacement mechanism.

Environment plugin options can explicitly override core navigator values: `hardware_concurrency`,
`max_touch_points`, `vendor`, `product_sub`, and `pdf_viewer_enabled`. Unspecified
values continue to come from the profile. Plugin/MIME collections remain core-owned;
profile entries take precedence, with synthetic PDF entries when enabled and the
profile collection is empty. `deviceMemory` is absent by default; setting
`device_memory_gb` in plugin options or the `privacy` profile scope exposes the supplied value. `expose_global_privacy_control` enables the
`globalPrivacyControl` property, whose value is `global_privacy_control` (default false).

## Registering a canvas backend

The installer context includes `canvas.register(type, factory)`. The factory runs
with the canvas as `this` and receives the context options. Return a context or
`null`. Context names are normalized to lowercase. Later registrations replace
earlier factories for new canvases; existing contexts keep their identity. Cleanup
removes a registration only if that factory is still current. Core returns null
for a different context type once a canvas has selected its context.

```javascript
const plugin = new Plugin('custom-canvas', `({canvas}) => {
  canvas.register('custom', function(options) {
    return {canvas: this, options};
  });
}`);
```

## Bundled behavior

| Plugin | Functional | Mocked or unavailable |
| --- | --- | --- |
| `plugins.host_storage(...)` / `hostStoragePlugin(...)` | Async hostStorage, origin-partitioned atomic batches, session-memory default or custom backend | Explicit extension; does not implement localStorage or IndexedDB; durability depends on the backend |
| `plugins.canvas(...)` / `canvasPlugin(...)` | Rendering contexts, state, transforms, solid fill/stroke/clear painting, real PNG encoding, configured dimensions | Paths/gradients/images/text do not rasterize; hit tests false; no fonts or image decoding |
| `plugins.environment(...)` / `environmentPlugin(...)` | Navigator overrides and connection values | Configured values; no network monitoring |
| `plugins.permissions(...)` / `permissionsPlugin(...)` | Notifications, permission queries and credentials | Synthetic permission results; no UI or authentication |
| `plugins.devices(...)` / `devicesPlugin(...)` | Bluetooth, clipboard, geolocation, wake lock and sharing | Configured availability and callable placeholders; no device access |
| `plugins.media(...)` / `mediaPlugin(...)` | Media devices, streams, source buffers, session and encrypted-media payloads | Existing synthetic state and no-ops; no capture, decoding or playback |
| `plugins.workers(...)` / `workersPlugin(...)` | Worker and service-worker objects | Callable placeholders; no worker execution |
| `plugins.trusted_types(...)` / `trustedTypesPlugin(...)` | Trusted Types factory | Legacy placeholders; no policy enforcement |
| `plugins.extraction()` / `extractionPlugin()` | Defuddle extraction via `page.extract()` on the current DOM | Uses DOM/attributes/inline style heuristics; no fabricated layout or computed style |

Browser storage is available without plugins. `localStorage`, CacheStorage and
IndexedDB are partitioned by origin and shared by pages in one session. They survive
navigation. `sessionStorage` is partitioned by origin and page and survives that
page's navigation. Blank and opaque-origin pages instead use isolated realm-local
storage, as described below. Local storage changes
notify other same-origin pages through `storage` events.

Memory is the default. Node `createSession({storagePath: '/path/storage.json'})`
and Python `Session(storage_path='/path/storage.json')` persist localStorage,
CacheStorage and committed IndexedDB records. The parent directory must exist;
only one session may open a storage file at a time. Session storage and cookies are
not written to this file. `navigator.storage.persisted()` reports this configuration.
Blank and opaque-origin pages use isolated, in-memory storage for the current realm,
including IndexedDB and caches. It is discarded on navigation or close and never
written to the storage file. Their cookie reads are empty and cookie writes are
harmless no-ops. There is no OPFS backend or quota policy.

IndexedDB uses pinned `fake-indexeddb` for requests, cursors, indexes, structured
values and transactional rollback. The native backend serializes transactions
across pages and retains committed records. Version-change and blocked events are
implemented within a realm; cross-page version-change notifications are not yet
implemented. A transaction from a stale schema connection aborts with VersionError.
CacheStorage retains empty caches and response body bytes. CookieStore reads and
writes the session cookie jar, excluding HttpOnly cookies; automatic cookie change
events are not implemented. Incomplete cookie mutations and invalid cookie name/value
separators settle as no-ops; reads return null or an empty array when no cookie
matches. Real backend failures still reject. SQLTransaction retains its synthetic success-callback payload without executing SQL;
`openDatabase` remains an explicit undefined property.

Web Locks coordinate same-origin pages in the session. Native ownership releases
locks on navigation or close; JS handles callbacks, aborts and returned promises.
Callbacks run asynchronously through the runtime task queue. Abort signals can be
combined with ifAvailable or steal and release pending or held requests. Cancelling
a held request settles its request promise; it cannot stop already-running JS.
Geometry supports points, rectangles, quads and affine 2D matrices without layout.
Typed OM supports unit values and inline attributeStyleMap edits, without cascade
or general compound-unit algebra. Prioritized tasks share the native task queue
with timers; idle callbacks run behind ready tasks with a zero time budget. There
is no rendering-idle prediction or inherited continuation priority.

Canvas records solid `fillRect`, `strokeRect`, `clearRect` and `putImageData`
pixels into an RGBA buffer. `getImageData` reflects painted pixels; `toDataURL`
and `toBlob` encode a valid 8-bit RGBA PNG. Paths, gradients/patterns, shadows,
filters, images and text pixels do not rasterize, and `drawFocusIfNeeded` remains
an ignored-result call. State, transforms and ImageData allocation are functional.

Canvas `rgba` configures unpainted background pixels (default transparent);
out-of-bounds pixels remain transparent, and alpha-disabled contexts return opaque
pixels. `characterWidth` (Python `character_width`) is the fallback glyph advance;
`measureText` first uses the active profile's font metrics. Only `image/png` is
encoded; unsupported types return `data:,`. Zero-size canvases return `data:,`.
Readback and encoding are capped at 16 million pixels. Bitmap decoding rejects;
natural image sizes are zero. Canvas preserves the core image placeholders,
including the undefined-returning decode() no-op. It does not add document
screenshots or layout.

The [graphics/audio example](https://github.com/lexiforest/brimp/blob/main/examples/plugins/graphics-audio.js) illustrates
JavaScript state plus explicit async backend operations. It does not implement
browser-complete WebGL/WebGPU/WebAudio. The [Node host worker example](https://github.com/lexiforest/brimp/blob/main/examples/plugins/node-host-worker.mjs)
uses worker_threads and subscriptions through an explicit `HostWorker` extension.
The workers plugin exposes a Worker placeholder with local handlers and no-op
postMessage/terminate. It does not execute source, fetch a URL or start a thread.
SharedWorker, service-worker execution, and native coordinators are absent.
The workers plugin supplies a ServiceWorkerContainer placeholder, not worker execution.

The devices plugin exposes Clipboard, Geolocation and WakeLock; permissions owns
CredentialsContainer; media owns MediaCapabilities and MediaSession. Their heavy operations retain legacy
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
and bluetooth surfaces remain environment/devices plugin extensions absent in saved Safari.
TrustedTypePolicyFactory/trustedTypes in the trusted-types plugin copy the legacy placeholders:
policy creation, sink queries and type checks return undefined, defaultPolicy is
null, and emptyHTML/emptyScript are empty strings. No policies, branded trusted
values or CSP enforcement are provided. The Safari-absent getTypeMapping stays absent.
The media plugin also exposes MediaStream with a stable unique id, inactive
state and manually dispatched track handlers. Its track queries, clone and
mutation methods retain legacy undefined-returning placeholders; there are no
tracks or capture. SourceBuffer is an illegal-constructor interface only: its
methods are legacy no-ops and its property placeholders use empty strings/local
string assignments. There is no MediaSource factory, buffering or decoder.

## Additional browser surfaces

These browser subsystem installers are included in the default set. They can be
combined with core APIs, browser subsystem plugins and extraction without adding native engines.

| Python / Node installer | Functional behavior | Limits and deterministic behavior |
| --- | --- | --- |
| `graphics(...)` / `graphicsPlugin(...)` | Canvas WebGL/WebGL2 context identity, buffer/state bookkeeping, Safari constants and legacy callable placeholders; opt-in WebGPU buffer allocation, mapping and copying | Fixed RGBA readback; shaders report compilation/linking failure; additional GL methods return undefined without effects; WebGPU execution rejects; no rasterization or native GPU |
| `audio(...)` / `audioPlugin(...)` | AudioBuffer channel data, graph connections, listener/compressor and Gain/Oscillator/Analyser state, AudioContext lifecycle and realm-clock time | Silent analysis; missing DSP factories, automation ramps and worklet loading are callable no-ops; decoding rejects; offline rendering returns a silent buffer, with no processing/output |
| `rtc()` / `rtcPlugin()` | Local peer/data-channel objects, description and candidate payload storage, RTC event payloads/handlers | No connection or media transport; legacy no-op operations and a permanently pending createOffer promise |

The RTC plugin is included in the default browser set. Use `bundled::rtc()`,
`rtcPlugin()` or `plugins.rtc()` when selecting an explicit set. Peer and
channel defaults use the saved Safari reference; `getConfiguration()` returns the
profile ICE server list. Sending, closing, negotiation and receiver/statistics
calls remain legacy no-ops returning undefined. close does not transition state. createOffer never settles and does
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
validation. speechSynthesis exposes the profile's captured voice list and
publishes it asynchronously with `voiceschanged`, as browsers do; the utterance
queue and audio output remain local placeholders.
The canvas plugin adds legacy captureStream/transferControlToOffscreen no-ops.
Path2D calls retain validation and opaque path identities without rendering.
Existing toBlob/toDataURL encode a real PNG for painted rectangles and
putImageData, and createImageBitmap supplies mock dimensions; paths, images and
text do not imply image codecs or rasterization.

The graphics plugin exposes WEBGL_debug_renderer_info with a stable extension
object, Safari's two constants and the active profile's vendor/renderer/unmasked
strings. It reports profile identity, not hardware. Its other WebGL constants use
Safari values and readonly descriptors. `getSupportedExtensions` returns the
profile list; `getParameter` returns profile identity, named limits and dynamic
viewport/clear-color state. The additional
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
WebGPU conformance implementation. Canvas and graphics may be installed in either order. Core `getContext`
selects a registered backend and retains one context type per canvas. Audio sample rate and output latency come from the active profile (Safari 27:
44100 Hz, 0.01 s); explicit plugin options override their corresponding values. Buffers
are capped at 16 million samples and analyser data remains deterministic silence.

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
coverage. Crypto is [core](/web-platform/), with an explicit supported algorithm
set. Worker/SharedWorker/service-worker execution, full SVG/CSSOM, media codecs
and WebRTC remain outside core; use explicit host plugins where needed.

## File loading, options and state lifetime

Node: `Plugin.fromFile(pathOrURL, {name, options, handlers, setup})`.
Python: `Plugin.from_file(path, name=..., options=..., handlers=..., setup=...)`.
The file must evaluate to an installer; bundle module dependencies into that
installer first (for example with esbuild). Use a named function declaration for
file-based installers so formatting keeps them valid when the loader evaluates
them as function expressions. Original file URLs are retained in
host-side installation diagnostics; evaluated installers use a neutral `web-api`
source label (the engine may omit it from stacks). Options
are serialized into each realm. [realm-library.js](https://github.com/lexiforest/brimp/blob/main/examples/plugins/realm-library.js)
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
boundary is not a sandbox for a malicious plugin. See [Browser support](/web-platform/)
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

The [Node](https://github.com/lexiforest/brimp/blob/main/examples/plugins/session-preferences.mjs) and
[Python](https://github.com/lexiforest/brimp/blob/main/examples/plugins/session_preferences.py) examples implement an async,
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

[SQLiteStorage](https://github.com/lexiforest/brimp/blob/main/examples/plugins/sqlite_storage.py) is a runnable Python backend
example using explicit BEGIN IMMEDIATE/COMMIT/ROLLBACK, native-origin partitions
and cross-connection coordination. Supply its path explicitly to opt into disk
persistence. It accepts JSON values; the default memory backend also accepts
copied binary values. It has tests for rollback, independent origins, visibility
across connections and reopen.

This extension is separate from core localStorage, sessionStorage and IndexedDB.
Use it for application-specific host storage operations and backend integrations.
