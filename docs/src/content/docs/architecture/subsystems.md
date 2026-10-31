---
title: Subsystem responsibilities
description: Decide where browser behavior and native resources belong in Brimp.
---

This page is for contributors changing the runtime. To select or write a plugin
for an application, start with the [plugin tutorial](/examples/plugins/) and
[usage guide](/guides/plugins/). For API coverage, use [Browser support](/web-platform/).

## Core runtime and plugins

Core provides the document and infrastructure used by every page. Plugins provide
browser subsystems that a session can select or replace.

| Core | Plugins |
| --- | --- |
| DOM parsing, mutation, events and shared lifecycle | Audio, graphics, RTC, device services and other browser subsystems |
| Script loading, scheduling, Fetch and shared storage | Subsystem constructors, state, placeholders and backend integration |
| Page and realm ownership | Per-realm installers and cleanup; optional host handlers |

Core versus plugin is independent of Rust versus JavaScript. Both layers can
use JavaScript implementations backed by native code.

Keep ownership, default activation, and backend capability separate. A plugin
can be enabled by default even when it supplies synthetic behavior rather than
a device backend. Core currently owns geometry, Typed OM, ImageData, navigator
identity, Web Locks, and browser storage. MediaEncryptedEvent belongs to media;
SpeechSynthesisUtterance belongs to audio. Canvas context selection is core,
while plugins register the contexts they implement.

## Preserve the calls browser scripts depend on

External page scripts may call an API and discard its result. A placeholder can
therefore be required for the script to continue. Replacing a harmless call with
a throw or rejection can break a workload even if the new result resembles a
stricter interpretation of the standard.

Keep each browser subsystem's constructors, state, placeholders and backend
integration together. Assess changes to return values, callbacks, Promise
behavior, and exposure against the scripts that use them. A missing backend
alone is not a reason to remove a callable API.

Plugins should reuse the runtime's DOM, scheduling, and lifecycle. Avoid a second
implementation of infrastructure already shared by pages.

## Rust and JavaScript responsibilities

JavaScript implements browser-facing objects, properties, coercion, callbacks,
and object identity. Rust connects these objects to native resources, owns their
lifetime and authoritative state, and enforces rules that page code must not
bypass.

| Area | JavaScript | Rust and native libraries |
| --- | --- | --- |
| DOM | Classes, properties, collections and browser behavior | Lexbor tree, parsing and mutations |
| Events and forms | Propagation, listeners, validation and control behavior | DOM access and host actions |
| Fetch and WebSocket | Browser objects, Promises and streams | Network I/O, cookies, cancellation and buffering |
| Timers | Public methods, callbacks and arguments | Clock, deadlines, scheduling and callback lifetime |
| WebCrypto | Validation, binary conversion and Promise interfaces | Key ownership, policy enforcement and BoringSSL operations |

For example, a CryptoKey wrapper holds a reference to a native key. Changing the
wrapper cannot grant permission to export a non-extractable key.

When adding behavior:

1. Keep authoritative state with its existing owner.
2. Put native access, lifetime management and trusted enforcement in Rust.
3. Use JavaScript for browser object behavior that can call those operations.
4. Reuse existing dependencies, and measure before moving work across the boundary.

## API exposure and private state

Before page scripts run, core and plugin APIs receive the runtime's built-in
function-source representation. Initialization preserves property flags and
object identity and does not invoke getters. Page scripts can still replace
mutable methods and add globals afterward.

Keep runtime controls in native-held functions or private state. The private
`__brimp*` properties must not be exposed to external page scripts. Plugin file
URLs and raw host exception details belong in host diagnostics. Plugins remain
trusted code and must deliberately choose what they expose.

For the observed page boundary, see [Window and environment](/web-platform/environment/#page-and-host-boundary).
For the files implementing these responsibilities, see
[Follow a page through the runtime](/architecture/javascript-runtime/#follow-the-implementation).
