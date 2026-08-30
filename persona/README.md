# Brimp persona schema and runtime support

[`schema.json`](schema.json) is the authoritative version-1 JSON Schema.
[`example.json`](example.json) is a complete configuration. Unknown fields are
rejected by the Rust types, and `schema_version` is required when loading JSON.
All schema fields are parsed, validated where they have cross-field invariants,
and resolved deterministically by `PersonaConfig::resolve()`.

Runtime support is a separate question: a resolved value is only considered
applied below when it changes an observable Brimp network or JavaScript surface.
Schema-only fields remain available for future implementation but currently do
not alter browsing behavior.

## Loading

- Rust: `PersonaConfig::from_json`, `from_json_file`, or `load_default_json`.
- CLI: pass `--persona PATH` to `brimp get` or `brimp crawl`.
- Python: pass the JSON text as `persona_json=`.
- Node: pass the JSON text as `createSession({ personaJson })`.
- `BRIMP_PERSONA_JSON` selects the opt-in default JSON path used by
  `load_default_json`; creating a browser does not implicitly read a file.

`transport.impersonation_profile` is the only transport selector. The obsolete
`browser` alias and navigation-policy `startup_url` field were intentionally not
migrated.

## Applied fields

| Group | Runtime-applied fields | Schema-only or limitations |
| --- | --- | --- |
| `schema_version` | Validated as version 1. | None. |
| `preset` | Supplies defaults for every resolved field; applied surfaces use those defaults. | Defaults for schema-only surfaces remain unapplied. |
| `seed` | Deterministically derives resolved sub-seeds. | Seeded canvas, graphics, audio, font, DOMRect, and SVG effects are not applied. |
| `identity` | `brand`, `version`, `platform`, `platform_version`, `architecture`, `bitness`, and `model` feed `navigator.userAgentData`. | `family` and `engine` are resolved metadata only. Identity does not synthesize a User-Agent; set `network.user_agent` explicitly. |
| `transport` | `impersonation_profile` selects the curl-impersonate HTTP/TLS profile for the native loader. | A caller-supplied `ResourceLoader` is responsible for its own transport fingerprint. |
| `network` | All fields: `user_agent`, `accept`, `accept_language`, `accept_encoding`, `sec_ch_ua`, `sec_ch_ua_mobile`, `sec_ch_ua_platform`, `sec_ch_ua_full_version`, `sec_ch_ua_full_version_list`, `sec_ch_ua_arch`, `sec_ch_ua_bitness`, `sec_ch_ua_platform_version`, and `sec_ch_ua_model`. They are installed on navigation, redirects, subresources, and fetches unless the individual request overrides a header. | None. |
| `features` | `user_agent_data`, `device_memory`, `network_information`, `permissions`, `bluetooth`, `notifications`, `geolocation`, `battery`, and `webrtc` gate their corresponding JavaScript surfaces. | Other feature flags remain schema-only. |
| `viewport` | `width`, `height`, and `device_scale_factor` drive layout, `innerWidth`, `innerHeight`, and `devicePixelRatio`. | Device scale factor is an integer in schema version 1. |
| `screen` | All fields: dimensions, available geometry, offsets, depth, extension state, and orientation type/angle. | The object is a Brimp-owned snapshot; screen-change events are not implemented. |
| `window` | `outer_width`, `outer_height`, `screen_x`, and `screen_y` expose outer and screen geometry. | Geometry is fixed for the page lifetime unless a new persona is installed. |
| `plugins` | `pdf_enabled` feeds `navigator.pdfViewerEnabled`; `entries` supply array-like `navigator.plugins` and `navigator.mimeTypes` snapshots with numeric, `item()`, and `namedItem()` lookup. | `block_system_entries` has no additional effect because Brimp has no system plugin inventory to merge. |
| `chrome` | `exposed`, `enumerable`, `runtime`, `app`, `load_times`, and `csi` control the basic `window.chrome` shape. | `window_key_strategy` and Chrome method return payloads are not applied. |
| `navigator` | All fields except `offscreen_canvas_enabled`. Notification permission, permission exposure, Bluetooth availability, and media-device exposure feed the emulated APIs described below. | Permission queries other than notifications return `granted`; media-device counts produce deterministic synthetic identifiers and labels. |
| `automation` | `webdriver` controls `navigator.webdriver`. | `expose_webdriver_helpers` is resolved only; Brimp does not install automation globals. |
| Top-level locale fields | `locale` and `languages` control Navigator values; `accept_language` controls the HTTP header and participates in locale resolution. | `locale` does not alter JavaScriptCore's Intl locale. `timezone` is resolved but cannot alter JavaScriptCore's Intl/Date timezone without a JSC change. |

## Emulated structured APIs

These APIs expose deterministic persona values without pretending to provide an
underlying device or service:

| Group | Emulated JavaScript behavior | Deliberate boundary |
| --- | --- | --- |
| `media` | `navigator.mediaDevices.enumerateDevices()`, `MediaSource.isTypeSupported()`, and `navigator.mediaCapabilities.decodingInfo()` return configured device counts, MIME support, and decoding capabilities. | Capture/playback and media pipelines are not implemented. |
| `speech` | `speechSynthesis.getVoices()` returns configured voice records. | Speech playback methods are inert. |
| `geolocation` | `getCurrentPosition()` and `watchPosition()` return configured coordinates; missing coordinates produce an unavailable error. | There is no permission prompt or location provider. |
| `webrtc` | `RTCRtpSender.getCapabilities()` and `RTCRtpReceiver.getCapabilities()` return configured audio/video codec records. | Peer connections, ICE, and SDP are not implemented. |
| `battery` | `navigator.getBattery()` resolves to the configured battery snapshot. | Values do not change and events are inert. |
| `storage` | `navigator.storage.estimate()` and legacy quota callbacks return configured usage and quota values. | This does not change actual storage allocation or persistence. |
| Navigator permissions | `Notification.permission`, `Notification.requestPermission()`, `navigator.permissions.query()`, and `navigator.bluetooth.getAvailability()` return persona-backed values. | There are no prompts, device discovery, or permission persistence. |

These are API-shape emulations. They are suitable for deterministic feature and
fingerprint probes, but they do not claim full WebIDL or WPT conformance. Values
are sourced from `ResolvedPersona`; presets or explicit schema fields can
customize them without changing the installer.

## Schema-only groups

These groups are fully parsed and resolved but do not currently change an
observable Brimp surface:

| Group | Why it is not applied in this repository-only stage |
| --- | --- |
| `graphics` | WebGL/WebGPU contexts and parameter interception are not implemented. |
| `css` | Prefix exposure and input/media feature overrides require Stylo/Blitz integration. |
| `audio` | Web Audio fingerprint hooks are absent. |
| `fonts` | Arbitrary font inventory and metric substitution require real font assets and Blitz text integration. |
| `canvas` | Canvas fingerprint-noise hooks are absent. |
| `domrect` | Quantization and transform-model changes require Blitz layout/geometry changes. |
| `engine` | Stack syntax, timing, errors, and builtin source text require JavaScriptCore changes, which are out of scope. |
| `svg` | Persona-specific SVG geometry requires Blitz/SVG geometry integration. |
| `native_functions` | Function source, descriptors, constructors, and stack sanitation require JavaScriptCore changes. |
| `noise` | This is an umbrella for the unsupported canvas, WebGL, audio, font, DOMRect, and SVG hooks. |

No Blitz or JavaScriptCore dependency is patched by this migration. When an
unsupported field is supplied, it still participates in validated resolution
and serialization, but it has no runtime effect; this is deliberate and should
not be mistaken for fingerprint coverage. The remaining schema-only groups are
closed as intentionally out of scope rather than deferred implementation work;
they should not be revisited unless the product requirements change.
