---
title: Window and environment
description: Profiles, navigation, timers, media queries and browser-service limits.
---

The browser profile supplies identity, locale, screen and device-related values.
Interface defaults represent a Safari-like environment. Supplied profiles can
change those values coherently, including values [captured from a real browser](/guides/profiles/).
Captured values are fixed observations; they do not connect Brimp to live browser devices.

| Area | What to expect |
| --- | --- |
| Navigation and history | URL changes, cloned history state and supported traversal events; no cross-document history/bfcache |
| Screen and viewport | Profile values, without layout or automatic resize/orientation events |
| Media queries and fonts | Profile-based answers and local metadata, without font loading or shaping |
| Time | Realm clocks, timers, prioritized tasks and idle callbacks; synthetic legacy timing |
| Storage and locks | Session/origin storage, page sessionStorage, cookie-jar access and coordinated Web Locks |
| Browser UI and playback | UI calls are no-ops; media playback is unavailable |

## Storage and shared state

Storage APIs are core. Same-origin pages in a session share localStorage,
IndexedDB, caches and locks. sessionStorage stays with its page. These stores
survive navigation; an optional session storage file persists localStorage,
IndexedDB and caches across sessions. CookieStore uses the session cookie jar.
See [storage lifetimes and limits](/guides/plugins/#bundled-behavior) for the
configuration and current IndexedDB/CacheStorage subset.

ImageData remains a core DOM value object. MediaEncryptedEvent belongs to the
media plugin and SpeechSynthesisUtterance to audio. These payloads do not require
a playback or speech backend. Navigator identity and plugin/MIME collections come from the profile.

## Window and navigation

Window aliases and event handlers belong to their realm. Location uses the
network/navigation pipeline; same-document history retains state. `window.close()`
does not close the host Page.

<details>
<summary>Detailed behavior and edge cases</summary>

Window uses the same DOM event-handler mechanism for its global callbacks,
including media, pointer, drag, animation and device-event properties. Manual
dispatch works without adding device, rendering or media event producers.
Unqualified window globals use the current realm; iframe window receivers retain
their own handler/state identity. name/status store strings, clientInformation
aliases navigator, origin follows location, and frameElement identifies an iframe
(null at the top level). closed is a false placeholder; UI close does not close
the runtime page. Prefixed animation-frame methods use the existing realm-clock
scheduling and cancellation.

Location uses the existing URL parser and navigation pipeline for component
writes, href/assign/replace/reload, and window.location/document.location
assignment. The top-level Location object keeps its identity during same-document
navigation. History retains cloned state, same-origin validation and asynchronous
back/forward/go traversal with events; scrollRestoration only stores a preference
and go(0) remains a no-op. Cross-document history/bfcache and full iframe Location
semantics are not implemented. ancestorOrigins is a stable empty DOMStringList
facade without frame-origin tracking. Detached Document.location remains null.

</details>

## Identity, screen and viewport

Navigator, screen and viewport values come from the active profile. Browser services belong to subsystem plugins enabled by default. Geometry values do not change in response to no-op
resize or scroll calls.

<details>
<summary>Detailed behavior and edge cases</summary>

Core Screen, ScreenOrientation, VisualViewport and BarProp read the active
BrowserProfile. The interface defaults supply 800×600 viewport, 800×652
outer window, device pixel ratio 2 and 1710×1107 screen (available 1710×1019 at
0,34); a supplied profile changes them. Window screen position is 110,34;
scroll/viewport offsets stay zero, scale is one, bars are visible and orientation
is landscape-primary at zero degrees. These are fixed profile values, not live
measurements during runtime execution. Resize/scroll calls do not change them; orientation/viewport
handlers support manual events only. Safari-absent orientation lock/unlock stay
absent. isSecureContext retains the legacy true placeholder without computing a
security context; styleMedia is undefined.

Navigator's common identity metadata stays in core: appCodeName/appName/product
and vendorSub use the saved Safari constants, and javaEnabled returns false.
Session-configured user agent, language and platform values remain unchanged.
sendBeacon is an explicit legacy no-op returning undefined without sending data.
Device and clipboard surfaces belong to devicesPlugin; credentials and permission
queries belong to permissionsPlugin. Media and worker surfaces belong to their
respective plugins. Their placeholders are part of each subsystem’s contract.

</details>

## Media queries

`matchMedia()` returns a MediaQueryList and evaluates supported queries against
profile viewport, media, color and interaction values. Listener methods and
`onchange` are exposed, but live media-query change events are not implemented.

## Fonts

`document.fonts` is a FontFaceSet. It can retain added FontFace objects and answer
font checks using profile families. `load()` and `ready` resolve metadata state;
they do not download fonts, shape text or perform layout. Canvas text metrics
use the canvas plugin’s profile data; that plugin is enabled by default.

## Performance and timers

Timers and performance entries use the realm clock. Legacy navigation timing
is a stable profile-derived snapshot, not measured DNS, connection, rendering or
load latency. See [Diagnostics and time](/guides/diagnostics/#logical-time) to
control the logical clock.

<details>
<summary>Detailed behavior and edge cases</summary>

Performance marks, measures, entry queries and observers use the realm clock and
existing local entry storage. Resource-timing control methods are legacy no-ops;
eventCounts is an empty-string placeholder. performance.navigation is a stable
synthetic object with type/redirectCount zero and standard numeric constants;
it does not measure redirects or history navigations. PerformancePaintTiming is
an interface-only placeholder; there are no paint entries. The resource-buffer
handler supports manual event dispatch, with no automatic buffer-full events.
Legacy performance.timing exposes a stable synthetic snapshot:
navigationStart uses the realm time origin; other phases use configured profile
offsets or zero. The snapshot does not change as logical time advances. It does not measure DNS, connection, rendering or load phases.

</details>

## Browser UI and media playback

Dialog, window-management and rendering-related methods generally perform no
work. Media elements retain attributes and local state, but `play()` rejects
with NotSupportedError and video dimensions remain zero.

<details>
<summary>Detailed behavior and edge cases</summary>

Window alert/confirm/prompt, open/close/focus/blur, find/print, move/resize/scroll,
captureEvents/releaseEvents, stop, reportError and getComputedStyle
are explicit legacy no-ops returning undefined. They do not open UI, navigate,
report errors, create computed-style objects or change DOM/layout state. These
contracts support ignored-result calls; no browser UI or computed-style backend
is implied by their presence.

Audio/video elements remain core DOM interfaces: attributes, URL reflection,
cloning, event handlers and local playback-property state use existing primitives.
Without a media controller, currentSrc stays empty, networkState/readyState are
zero, duration is NaN, and play rejects with NotSupportedError. srcObject stores
the assigned value without processing a stream; preservesPitch stores a boolean.
canPlayType retains a limited audio MIME classification, not decoder detection.
Buffering, track and remote-playback objects (buffered, played, seekable,
textTracks and remote) are explicit legacy empty-string placeholders. Track/sink,
video-frame, quality, fullscreen and picture-in-picture methods are no-ops returning
undefined, without callbacks, UI or promises. Video dimensions stay zero and
fullscreen flags stay false. There is no new playback, codec or device backend.


</details>

## Page and host boundary

Page scripts run in a JavaScriptCore realm without Node/Python host globals.
Host callbacks cross an explicit value/reference bridge. This is not a claim
of complete security isolation or browser indistinguishability.

<details>
<summary>Detailed behavior and edge cases</summary>

Each page executes in its own JavaScriptCore realm. The Node/Python host objects
are not installed there: `process`, `global`, `Buffer`, `require`, `module`,
`__filename` and `__dirname` are absent from the page global by default. Plugin
host calls cross the copied-value bridge, not a shared host object graph.

Script scheduling, currentScript control, module loading and frame-index
bookkeeping use native-held realm functions and private state throughout
navigation and module execution.
The bundled core-js shared store stays inside its bundle. Core and plugin APIs
share initialization-time source finalization; plugin file paths and raw host
exceptions remain host diagnostics. See [the plugin contract](/guides/plugins/).

There is no global-name filtering or freeze applied to application code. Pages
can create properties (even names resembling internals), replace mutable APIs,
and load their own dependencies. Trusted plugins control their intentional
public exports. This boundary does not claim full security isolation or
human-browser indistinguishability: engine behavior, module compilation,
unsupported APIs, optional mocks and network/interaction behavior remain
observable. Detector findings do not expand the core contract.

</details>


## Verification

The [runtime tests](https://github.com/lexiforest/brimp/tree/main/tests/runtime/environment/)
record the behavior behind this reference. Safari comparisons cover the recorded
cases; they do not establish complete browser conformance. The
[field ledger](https://github.com/lexiforest/brimp/blob/main/COVERAGE.yaml) tracks
individual decisions and evidence.
