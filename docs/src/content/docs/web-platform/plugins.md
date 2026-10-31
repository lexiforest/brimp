---
title: Browser plugins
description: Behavior and limits of the bundled browser-service, graphics, audio and RTC APIs.
---

Use this reference to check what a bundled plugin's browser APIs do. For choosing
plugins or connecting application functions, use the [plugin guide](/guides/plugins/)
and [tutorial](/examples/plugins/).

The default set includes environment, permissions, devices, media, workers,
Trusted Types, canvas, graphics, audio, and RTC. Extraction and host storage are
selected separately. The tables distinguish useful state and operations from
placeholders that let page scripts continue without a device or browser engine.

## Browser services and extraction

| Plugin | Functional | Mocked or unavailable |
| --- | --- | --- |
| `plugins.host_storage(...)` / `hostStoragePlugin(...)` | Async hostStorage, origin-partitioned atomic batches, session-memory default or custom backend | Explicit extension; does not implement localStorage or IndexedDB; durability depends on the backend |
| `plugins.canvas(...)` / `canvasPlugin(...)` | Rendering contexts, state, transforms, solid fill/stroke/clear painting, real PNG encoding, configured dimensions | Paths/gradients/images/text do not rasterize; hit tests false; no fonts or image decoding |
| `plugins.environment(...)` / `environmentPlugin(...)` | Navigator overrides and connection values | Configured values; no network monitoring |
| `plugins.permissions(...)` / `permissionsPlugin(...)` | Notifications, permission queries and credentials | Synthetic permission results; no UI or authentication |
| `plugins.devices(...)` / `devicesPlugin(...)` | Bluetooth, clipboard, geolocation, wake lock and sharing | Configured availability and callable placeholders; no device access |
| `plugins.media(...)` / `mediaPlugin(...)` | Media devices, streams, source buffers, session and encrypted-media payloads | Synthetic state and no-ops; no capture, decoding or playback |
| `plugins.workers(...)` / `workersPlugin(...)` | Worker and service-worker objects | Callable placeholders; no worker execution |
| `plugins.trusted_types(...)` / `trustedTypesPlugin(...)` | Trusted Types factory | Legacy placeholders; no policy enforcement |
| `plugins.extraction()` / `extractionPlugin()` | Defuddle extraction via `page.extract()` on the current DOM | Uses DOM/attributes/inline style heuristics; no layout or computed style |

## Graphics, audio and RTC

| Python / Node installer | Functional behavior | Limits and deterministic behavior |
| --- | --- | --- |
| `graphics(...)` / `graphicsPlugin(...)` | Canvas WebGL/WebGL2 context identity, buffer/state bookkeeping, Safari constants and legacy callable placeholders; opt-in WebGPU buffer allocation, mapping and copying | Fixed RGBA readback; shaders report compilation/linking failure; additional GL methods return undefined without effects; WebGPU execution rejects; no rasterization or native GPU |
| `audio(...)` / `audioPlugin(...)` | AudioBuffer channel data, graph connections, listener/compressor and Gain/Oscillator/Analyser state, AudioContext lifecycle and realm-clock time | Silent analysis; missing DSP factories, automation ramps and worklet loading are callable no-ops; decoding rejects; offline rendering returns a silent buffer, with no processing/output |
| `rtc()` / `rtcPlugin()` | Local peer/data-channel objects, description and candidate payload storage, RTC event payloads/handlers | No connection or media transport; legacy no-op operations and a permanently pending createOffer promise |

## Navigator values

Environment plugin options override these navigator values: `hardware_concurrency`,
`max_touch_points`, `vendor`, `product_sub`, and `pdf_viewer_enabled`. Unspecified
values continue to come from the profile. Plugin/MIME collections remain core-owned;
profile entries take precedence, with synthetic PDF entries when enabled and the
profile collection is empty. `deviceMemory` is absent by default; setting
`device_memory_gb` in plugin options or the `privacy` profile scope exposes the supplied value. `expose_global_privacy_control` enables the
`globalPrivacyControl` property, whose value is `global_privacy_control` (default false).

## Canvas

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

The canvas plugin adds legacy captureStream/transferControlToOffscreen no-ops.
Path2D calls retain validation and opaque path identities without rendering.
Existing toBlob/toDataURL encode a real PNG for painted rectangles and
putImageData, and createImageBitmap supplies mock dimensions; paths, images and
text do not imply image codecs or rasterization.

## Permissions, devices and media services

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

## Workers

The [Node host worker example](https://github.com/lexiforest/brimp/blob/main/examples/plugins/node-host-worker.mjs)
uses worker_threads and subscriptions through an explicit `HostWorker` extension.
The workers plugin exposes a Worker placeholder with local handlers and no-op
postMessage/terminate. It does not execute source, fetch a URL or start a thread.
SharedWorker, service-worker execution, and native coordinators are absent.
The workers plugin supplies a ServiceWorkerContainer placeholder, not worker execution.

## Trusted Types

TrustedTypePolicyFactory/trustedTypes in the trusted-types plugin copy the legacy placeholders:
policy creation, sink queries and type checks return undefined, defaultPolicy is
null, and emptyHTML/emptyScript are empty strings. No policies, branded trusted
values or CSP enforcement are provided. The Safari-absent getTypeMapping stays absent.

## Media streams and source buffers

The media plugin also exposes MediaStream with a stable unique id, inactive
state and manually dispatched track handlers. Its track queries, clone and
mutation methods retain legacy undefined-returning placeholders; there are no
tracks or capture. SourceBuffer is an illegal-constructor interface only: its
methods are legacy no-ops and its property placeholders use empty strings/local
string assignments. There is no MediaSource factory, buffering or decoder.

## RTC

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

## Audio

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

## Graphics

The graphics plugin accepts `rgba`, `renderer`, and `webgpu` (default false).

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

## Connect a graphics or audio backend

The [graphics/audio example](https://github.com/lexiforest/brimp/blob/main/examples/plugins/graphics-audio.js) illustrates
JavaScript state plus explicit async backend operations. It does not implement
browser-complete WebGL/WebGPU/WebAudio.

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
