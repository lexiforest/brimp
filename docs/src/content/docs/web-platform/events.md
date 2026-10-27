---
title: Events
description: DOM dispatch, handlers and input payloads without device or UI backends.
---

Events use the DOM propagation model: capture, target and bubble phases,
cancelation, once/passive/signal listeners, and Shadow DOM retargeting.

An event class or handler property can exist without a producer for that event.
Scripts can manually dispatch many device, media and UI events, but Brimp does
not add the corresponding hardware, renderer or browser interface.

## Dispatch and handlers

Listener identity, ordering and cancellation are retained. Script redispatch
clears trusted status. Mutation observation uses MutationObserver; MutationEvent
is absent.

<details>
<summary>Detailed behavior and edge cases</summary>

Event has readonly state, immutable phase constants, a nonconfigurable per-instance
isTrusted getter and timestamps relative to the runtime clock. Dispatch supports
capture/target/bubble phases, cancellation, passive/once/signal listeners and
shadow retargeting. Propagation flags reset after dispatch; redispatching an active
or uninitialized legacy event throws InvalidStateError. Script redispatch clears
trusted status. EventTarget validates listener arguments and matches registrations
by type, callback and capture. The nonstandard Event.path field stays absent,
matching Safari; composedPath provides paths during dispatch.

CustomEvent keeps its detail payload by identity behind a readonly accessor.
UIEvent exposes readonly view, signed 32-bit detail and legacy which values;
view accepts a Window or null. Both support legacy initialization outside active
dispatch and validate required types and dictionary values. UIEvent's nonstandard
sourceCapabilities field is absent, matching Safari. Mouse and keyboard payload
details are separate contracts.

TextEvent is created through document.createEvent('TextEvent'), inherits UIEvent,
and supports readonly text payloads initialized with initTextEvent outside active
dispatch. Its direct constructor is illegal. MutationEvent is absent, as in Safari;
DOM mutation observation uses MutationObserver.

Body and frameset ordinary window-event accessors share their owning window's
handler state, including detached elements and same-origin iframe documents.
Adoption changes the target window; documents without a window expose null and
ignore assignments. Safari's distinct HTMLElement accessors still store local
handler state when called explicitly on a body or frameset. Shared tests cover
44 properties, assignment conversion, descriptors and receiver checks, cloning,
adoption, listener order, explicit dispatch and return-false cancellation.
Window ErrorEvent callbacks receive five canonical payload arguments, use an
undefined callback receiver (a non-strict function substitutes its own global),
and cancel only on strict true. Ordinary error events retain one argument and
strict-false cancellation. ErrorEvent preserves message UTF-16, converts filename
to a scalar-value string, and follows Safari's undefined default error payload.
BeforeUnloadEvent is created through document.createEvent and must be initialized
before dispatch. A non-null handler return is string-converted, cancels the event,
and populates an empty returnValue; existing strings are preserved. For ordinary
Event objects, beforeunload handler returns are ignored without conversion.
The 520-case Safari corpus covers both algorithms, including iframe windows.
Inline handler compilation and browser generation of error, print, network,
navigation and rejection events are separate contracts. Before-unload registration
and dispatch do not add navigation confirmation UI.

</details>

## Mouse and keyboard payloads

Mouse and keyboard events expose readonly payloads and modifier state.
Layout-dependent coordinates are fixed zero placeholders.

<details>
<summary>Detailed behavior and edge cases</summary>

MouseEvent and KeyboardEvent expose readonly payloads and modifier state, including
legacy initialization. Mouse client/screen coordinates follow Safari's ordinary
integer reads; buttons and keyboard location/key codes use their integer types.
Synthetic KeyboardEvent.which remains zero, while runtime input events retain
their generated key codes. Keyboard location constants are immutable on both the
constructor and prototype. Mouse fromElement/toElement follow enter/leave targets.
Mouse pageX/pageY, offsetX/offsetY and layerX/layerY are explicit legacy zero
placeholders in core: they do not calculate scroll, layout or layer geometry.

</details>

## Pointer payloads

PointerEvent supports device payloads and ordinary tilt/angle conversion.
Coalesced/predicted sample methods return `undefined`; no hardware sampling
or prediction is performed.

<details>
<summary>Detailed behavior and edge cases</summary>

PointerEvent has readonly device payloads and ordinary tilt/angle conversion,
with MouseEvent dispatch and modifiers. Representative Safari checks compare
orientation numerically, without requiring bit-identical trigonometric rounding.
getCoalescedEvents and getPredictedEvents are explicit callable legacy placeholders
returning undefined; they do not provide hardware samples or predictions. These
methods stay available in Brimp's core runtime without secure-context gating.

</details>


## Verification

The [runtime tests](https://github.com/lexiforest/brimp/tree/main/tests/runtime/events/)
record the behavior behind this reference. Safari comparisons cover the recorded
cases; they do not establish complete browser conformance. The
[field ledger](https://github.com/lexiforest/brimp/blob/main/COVERAGE.yaml) tracks
individual decisions and evidence.
