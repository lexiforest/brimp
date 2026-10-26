# Common web core

Brimp owns the canonical DOM, events, script execution, network primitives,
structured cloning and crypto. These are available in both Node and Python
without plugins. Specialized capabilities are explicit plugins; see [PLUGINS.md](PLUGINS.md).

The browser corpus exercises input handlers/activation, token lists,
NodeIterator, attribute removal, XPath and non-rendering Shadow DOM. Shadow roots
have isolated canonical subtrees, named slots, connected roots and composed event
retargeting, including closed-root path filtering. Custom element definitions
read observedAttributes before construction, as required by Lit's initialization.
There is no CSS cascade, computed geometry or rendering. Manual slot assignment,
declarative shadow roots and full focus delegation remain
outside this implementation. XPath uses the pinned `xpath` library; mutation
invalidation of XPath iterators is not yet implemented.

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

MouseEvent and KeyboardEvent expose readonly payloads and modifier state, including
legacy initialization. Mouse client/screen coordinates follow Safari's ordinary
integer reads; buttons and keyboard location/key codes use their integer types.
Synthetic KeyboardEvent.which remains zero, while runtime input events retain
their generated key codes. Keyboard location constants are immutable on both the
constructor and prototype. Mouse fromElement/toElement follow enter/leave targets.
Mouse pageX/pageY, offsetX/offsetY and layerX/layerY are explicit legacy zero
placeholders in core: they do not calculate scroll, layout or layer geometry.

PointerEvent has readonly device payloads and ordinary tilt/angle conversion,
with MouseEvent dispatch and modifiers. Representative Safari checks compare
orientation numerically, without requiring bit-identical trigonometric rounding.
getCoalescedEvents and getPredictedEvents are explicit callable legacy placeholders
returning undefined; they do not provide hardware samples or predictions. These
methods stay available in Brimp's core runtime without secure-context gating.

Core Element and Document expose legacy callable placeholders for scrolling,
animation, hit testing, fullscreen, picture-in-picture exit, pointer capture/lock,
and document editing commands. They return undefined without changing DOM or UI
state, dispatching events, or providing promises, animations or geometry results.
The deprecated document captureEvents, releaseEvents and clear methods are also
no-ops. These contracts support scripts that discard the results; they do not
provide browser rendering or interactive editing. See dom-ui-placeholders.test.mjs
for the covered calls and an ignored-result DOM workflow.

Element client/scroll dimensions and border offsets are zero placeholders.
scrollTop/scrollLeft retain assigned finite numbers locally, without scrolling,
clamping to layout bounds or emitting events. computedStyleMap returns undefined;
the optional cssTypedOm attributeStyleMap still edits real inline declarations.
HTMLElement.isContentEditable follows contenteditable ancestors and the document
designMode flag, without implementing an editor. nonce is ordinary attribute
reflection, without CSP nonce hiding. outerText reads the existing raw innerText
fallback and replaces its element with text/BR nodes when assigned; rendered-text
whitespace/layout semantics are not implemented.

SVG elements share nonce, autofocus and tabIndex reflection, and expose their
nearest SVG ancestor through ownerSVGElement/viewportElement. className is a
stable baseVal/animVal facade backed by the class attribute; animVal mirrors the
base value without an animation engine. Common SVG graphics elements use the
SVGGraphicsElement interface; other SVG nodes retain generic SVGElement wrappers.
getBBox/getCTM/getScreenCTM are legacy no-ops returning undefined, while transform,
requiredExtensions and systemLanguage retain legacy empty-string placeholders.
There is no SVG renderer, matrix computation, animated transform list or full SVG
interface hierarchy. DOM attributes, styles, events and cloning continue to work.

Window uses the same DOM event-handler mechanism for its global callbacks,
including media, pointer, drag, animation and device-event properties. Manual
dispatch works without adding device, rendering or media event producers.
Unqualified window globals use the current realm; iframe window receivers retain
their own handler/state identity. name/status store strings, clientInformation
aliases navigator, origin follows location, and frameElement identifies an iframe
(null at the top level). closed is a false placeholder; UI close does not close
the runtime page. Prefixed animation-frame methods use the existing realm-clock
scheduling and cancellation.

Core Screen, ScreenOrientation, VisualViewport and BarProp expose a fixed profile
from the saved Safari fixture: 800×600 viewport, 800×652 outer window, device pixel
ratio 2 and 1710×1107 screen (available 1710×1019 at 0,34). Window screen position
is 110,34; scroll/viewport offsets stay zero, scale is one, bars are visible and
orientation is landscape-primary at zero degrees. These are explicit placeholders,
not measurements or persona-dependent settings. Resize/scroll calls do not change
them; orientation/viewport handlers support manual events only. Safari-absent
orientation lock/unlock stay absent. isSecureContext retains the legacy true
placeholder without computing a security context; styleMedia is undefined.

Location uses the existing URL parser and navigation pipeline for component
writes, href/assign/replace/reload, and window.location/document.location
assignment. The top-level Location object keeps its identity during same-document
navigation. History retains cloned state, same-origin validation and asynchronous
back/forward/go traversal with events; scrollRestoration only stores a preference
and go(0) remains a no-op. Cross-document history/bfcache and full iframe Location
semantics are not implemented. ancestorOrigins is a stable empty DOMStringList
facade without frame-origin tracking. Detached Document.location remains null.

Window alert/confirm/prompt, open/close/focus/blur, find/print, move/resize/scroll,
captureEvents/releaseEvents, stop, reportError, matchMedia and getComputedStyle
are explicit legacy no-ops returning undefined. They do not open UI, navigate,
report errors, create media-query/style objects or change DOM/layout state. These
contracts support ignored-result calls; no browser UI or computed-style backend
is implied by their presence.

Link and script metadata use the common attribute reflection, URL resolution and
token-list machinery. Links expose imageSizes/imageSrcset for preload metadata;
scripts expose referrerPolicy and text. Script text reads direct text children,
and dynamic script execution uses that same source. These properties do not add
image preload processing or new script scheduling modes.

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
See media-element-family.test.mjs for the supported workflows and limitations.

XMLHttpRequest uses the fetch pipeline for asynchronous requests, response state,
headers, body conversion, progress events, cancellation and timeout handling.
These are runtime request contracts, not a claim of complete browser XHR/CORS
conformance. Request bodies support text, JSON, byte buffers and blobs, cloning
before consumption and propagated abort-signal state. Headers normalize names
and expose sorted iteration. Request.formData parses URL-encoded bodies through
the existing URLSearchParams/FormData primitives; other formats, including
multipart, retain the legacy undefined-returning placeholder without consuming
the body. This does not add multipart parsing or full Fetch policy enforcement.

Performance marks, measures, entry queries and observers use the realm clock and
existing local entry storage. Resource-timing control methods are legacy no-ops;
eventCounts is an empty-string placeholder. performance.navigation is a stable
synthetic object with type/redirectCount zero and standard numeric constants;
it does not measure redirects or history navigations. PerformancePaintTiming is
an interface-only placeholder; there are no paint entries. The resource-buffer
handler supports manual event dispatch, with no automatic buffer-full events.
Legacy performance.timing exposes a stable synthetic snapshot:
navigationStart uses the realm time origin; other phase timestamps are zero or
that same origin. It does not measure DNS, connection, rendering or load phases.

CSS style properties use one shared surface generated from the saved Safari
inventory and the vendored Lexbor property registry. The 90 inventoried properties
supported by Lexbor retain parsed declaration values, style-attribute updates,
serialization and cloning. Another 443 names are explicit legacy placeholders:
they start as empty strings and store assigned JavaScript values on the style
object. Those values do not enter cssText, getPropertyValue, length, serialized
stylesheet rules or cloned DOM attributes; CSS.supports continues to report parser support.
The 42 inventoried names absent in Safari remain absent on a fresh style object.
This adds no layout, cascade or computed-style engine. Regenerate the name table
with `node scripts/coverage/css-properties.mjs` after changing the reference
inventory or vendored property registry.

CSS.registerProperty is a legacy callable placeholder returning undefined; it
does not register syntax, inheritance or initial values. CSS.supports/escape and
CSSStyleDeclaration mutation continue to use the existing selector/declaration
primitives. Numeric CSS factories and rectangle arithmetic remain in the optional
cssTypedOm and geometry plugins, without document measurement or rendering.

Navigator's common identity metadata stays in core: appCodeName/appName/product
and vendorSub use the saved Safari constants, and javaEnabled returns false.
Session-configured user agent, language and platform values remain unchanged.
sendBeacon is an explicit legacy no-op returning undefined without sending data.
Device, clipboard, credentials and other optional services belong to servicesPlugin.

## Page exposure boundary

Each page executes in its own JavaScriptCore realm. The Node/Python host objects
are not installed there: `process`, `global`, `Buffer`, `require`, `module`,
`__filename` and `__dirname` are absent from the page global by default. Plugin
host calls cross the copied-value bridge, not a shared host object graph.

Script scheduling, currentScript control, module loading and frame-index
bookkeeping use native-held realm functions and private state throughout
navigation and module execution.
The bundled core-js shared store stays inside its bundle. Core and plugin APIs
share initialization-time source finalization; plugin file paths and raw host
exceptions remain host diagnostics. See [the plugin contract](PLUGINS.md).

There is no global-name filtering or freeze applied to application code. Pages
can create properties (even names resembling internals), replace mutable APIs,
and load their own dependencies. Trusted plugins control their intentional
public exports. This boundary does not claim full security isolation or
human-browser indistinguishability: engine behavior, module compilation,
unsupported APIs, optional mocks and network/interaction behavior remain
observable. Detector findings do not expand the core contract.

## Crypto

`crypto.getRandomValues`, `randomUUID` and `crypto.subtle` are core. The same Rust
vendored OpenSSL backend serves both language bindings.
Implemented SubtleCrypto operations are Promise-returning, copy BufferSource
inputs, and enforce CryptoKey usages and extractability. Key bytes and authoritative usage/extractability policy live in native realm state.
JavaScript CryptoKey wrappers hold opaque handles; generated keys never pass their
bytes through JavaScript. Keys are released when the realm is destroyed.

| Algorithm | Operations and formats |
| --- | --- |
| SHA-1, SHA-256, SHA-384, SHA-512 | digest |
| HMAC with those hashes | generate/import/export, sign/verify; raw and oct JWK |
| AES-GCM/CBC/CTR | 128/192/256-bit keys; generate/import/export, encrypt/decrypt, wrap/unwrap; raw and oct JWK |
| AES-KW | RFC 3394 key wrapping/unwrapping; raw and oct JWK |
| RSASSA-PKCS1-v1_5, RSA-PSS | generate/import/export, sign/verify; SPKI, PKCS8, RSA JWK |
| RSA-OAEP | generate/import/export, encrypt/decrypt, wrap/unwrap; hash-bound keys and labels |
| ECDSA | P-256/P-384/P-521; raw public points, SPKI, PKCS8 and EC JWK; browser-format signatures |
| ECDH | P-256/P-384/P-521; deriveBits/deriveKey with matching public peer keys |
| HKDF, PBKDF2 | raw non-extractable base keys; deriveBits/deriveKey |

AES-GCM accepts nonempty IVs and browser tag lengths (32, 64, 96, 104, 112,
120, 128 bits); callers must supply a unique IV per key. AES-CBC uses PKCS#7
padding. AES-CTR respects the requested 1–128 counter bits, including rollover,
and rejects inputs that would reuse a counter. RSA generation accepts
1024–16384-bit moduli. RSA JWK private imports require all CRT components; EC
raw public imports require uncompressed points. Derived/generated secret material
stays native unless explicitly exported. Native checks bind key type, algorithm,
usages, extractability and peer curve; modifying JS metadata does not change them.

Ed25519/X25519, CryptoKey structured cloning and full WebCrypto conformance are
not implemented. Unsupported algorithms reject with NotSupportedError. Crypto
uses native synchronous operations behind Promise-returning Web APIs; expensive
key generation/derivation occupies the page owner thread. OpenSSL is statically
built from the Cargo-pinned source, with packaged license notices; no additional
OpenSSL runtime SDK is required.

The `jose` fixture checks HS256 signing and verification. The `jose-extended`
fixture checks RSA/PSS/ECDSA JWTs and RSA-OAEP/ECDH-AES-KW JWE, including damaged
signature/tag rejection.

## DOM lifecycle, ranges, files and streams

Navigation exposes loading -> interactive -> complete through document.readyState
and readystatechange, with DOMContentLoaded before complete and load. Range has
cross-node text, clone/extract/delete contents, insertion, surrounding and
contextual fragments, with boundary updates for DOM mutations, character-data
edits, splitText and normalize. These are non-rendering operations; no range
rectangles or layout measurements are supplied.

Collections retain identity and update after mutations. Document adoption, Attr
creation/attachment, namespace-aware attributes and queries, TreeWalker navigation,
and inert template owner documents use the canonical DOM. queueMicrotask and
MutationObserver delivery share JavaScriptCore's Promise job queue.
Document forms/images/embeds/plugins/links/scripts/anchors/applets are stable,
readonly HTMLCollections backed by the native document tree. Membership requires
the matching HTML namespace/local name and, for links and anchors, a null-namespace
href or name attribute (including empty values). Collections exclude template and
shadow contents, follow moves/adoption/document.write, and include a qualifying
document element. Plugins aliases embeds; applets stays empty as in Safari.
Named lookup prefers a matching ID over a matching name in Safari and reads
canonical attributes. HTMLCollection iteration uses the generic live Array values
iterator. These collections do not load resources or implement plugin/applet engines.
HTMLCollection uses private state and an internal factory; direct construction is
illegal. Its methods validate native identity and required arguments, convert
item indices to unsigned long and names to DOMString, then read live state.
Indexed and supported named entries are readonly, ordinary expandos are retained,
and named entries respect Safari's prototype collisions and own-key ordering.
Public item/length/name overrides cannot change internal indexed or named lookup.
Form control collections retain their subclass prototype through the same factory.
Freezing and non-extensibility are explicitly unsupported: Safari can add live
entries to a non-extensible collection, which JavaScript proxy invariants cannot
represent. Brimp rejects these operations and keeps the collection live.
Document, Element and DocumentFragment children/childElementCount/firstElementChild/
lastElementChild use native immediate element children in every namespace.
Children is a stable live HTMLCollection; the other getters read native state
without consulting public childNodes, children, length or item properties. Native
parent and child identity survives prototype changes. Text, comments and doctypes
are excluded; parsed template contents and shadow trees remain separate from
ordinary children. Moves, adoption, fragment insertion and document.write update
existing collections.
Element and CharacterData previousElementSibling/nextElementSibling traverse the
native sibling list, skipping text, comments and doctypes while accepting elements
in every namespace. They remain live across removal, reinsertion, adoption,
splitText and normalize, preserve template/shadow boundaries, and ignore public
sibling getters and JavaScript prototype changes. Shared tests cover supported
Text and Comment instances; this does not add XML parsing or CDATA/processing-
instruction creation APIs.
CharacterData data/length and substringData/appendData/insertData/deleteData/
replaceData use canonical UTF-16 data and Safari's argument, conversion and error
ordering. The data setter maps null to an empty string; method DOMString arguments
retain ordinary null/undefined conversion. Edits preserve lone surrogates, update
live ranges and mutation records even for no-op writes, and read state after
reentrant argument conversion. Public data/length/textContent overrides and changed
prototypes do not interfere with edits, observers, iterators, textarea selection
or stylesheet reactions. Tests cover Text and Comment creation paths without
adding XML parsing or CDATA/processing-instruction factories.

Text construction creates an ordinary detached text node. CharacterData itself
is not constructible. Text.wholeText joins adjacent text siblings without crossing
comments or elements. Text.splitText splits UTF-16 data, inserts the new sibling
when attached, and updates live ranges and mutation observers. Detached splits
leave ranges on the original node. Representative shared Safari tests cover
construction, adjacency, splitting, bounds errors and these DOM effects.

Attr exposes readonly names, namespace metadata, ownerElement and specified,
with enumerable Web IDL accessors. Its value setter converts DOMStrings and
updates the attached attribute, mutation observers and custom-element callbacks.
Removed attributes retain their last value and remain independent of replacement
attributes. Namespaced attribute writes, attachment and removal deliver the
namespace to attributeChangedCallback. Shared Safari tests cover these operations
plus detached values, cloning and adoption.

NodeList is a non-Array collection with readonly indexed entries and length;
scripts cannot construct it directly. childNodes and other live producers update
after DOM mutations, while querySelectorAll and mutation records retain snapshots.
item converts an unsigned index and returns null outside the list. entries, keys,
values, forEach and Symbol.iterator use generic Array functions as in Safari.
Freezing and preventing extensions are unsupported, matching the explicit limit
of Brimp's other proxy-backed live collections. Four representative shared tests
cover ordinary reads, enumeration, mutation, snapshots and iteration.

Node exposes the standard immutable node-kind/document-position constants and
readonly tree metadata. Direct Node construction is illegal; internal Attr and
custom-element construction remain supported. Child/sibling reads stay live
across insertion, removal and adoption. getRootNode accepts an optional dictionary
(including null) and crosses shadow roots only with composed:true; isConnected
follows the composed root. Representative Safari tests cover these ordinary
interfaces and document, fragment, text, comment, doctype, attribute and shadow
relationships.

Node.isSameNode compares identity; contains follows the ordinary parent tree
without treating attributes or shadow descendants as children of their owner or
host. isEqualNode compares node kinds, names, namespaces, attributes and descendant
data, independently of attribute order. These methods require a nullable Node
argument. Node.baseURI uses the owning document's first eligible HTML base href
and follows base edits and adoption, including detached nodes and Attr objects.
It shares native base selection and URL parsing with URL-valued attributes.

Node.textContent and nodeValue use nullable string conversion, including for
ignored document/doctype writes. Container text replacement updates mutation
records and live ranges; normalize merges adjacent text and removes empty text
without crossing comments. Clones have independent attributes and descendants,
including template content and the supported form control state; listeners and
expando properties are not copied. Ordinary shadow trees are not cloned, and
cloning a ShadowRoot directly throws NotSupportedError.

Node appendChild, insertBefore, replaceChild and removeChild validate required
Node arguments, parent membership and document hierarchy before changing the
tree. Insertion also rejects cycles through shadow hosts. Ordinary moves and
fragment insertions retain node identities, adopt descendants, update observers
and live ranges, and deliver custom-element connection callbacks. Fragment
contents receive insertion hooks even after the fragment itself is drained.

Node namespace lookup follows ancestor declarations, using the document element
for documents and the owner element for attached attributes. Fragments and shadow
roots do not inherit a host namespace context. Following installed Safari,
lookupPrefix can return an ancestor's declared prefix even when a descendant
rebinds that prefix. compareDocumentPosition includes attached attributes in
document ordering and provides stable reciprocal ordering for disconnected trees;
the direction chosen between disconnected trees is implementation-specific.

`document.doctype` returns the canonical DocumentType node or null. DocumentType
exposes readonly name/publicId/systemId, cloning, adoption and ChildNode methods.
Importing a DocumentType throws NotSupportedError, matching installed Safari.
Document compatibility mode comes from the parser and is retained when cloning;
removing the doctype does not change it. DOMParser mode selection follows
installed Safari's fast fragment eligibility, including supported tags/nesting,
attribute syntax, and text/depth limits. Lexbor continues to build the DOM.
The shared mode tests compare 606 inputs with measured Safari results; this
does not claim complete browser parsing conformance.

Forms expose association (including controls outside their form), labels, reset,
constraint validity/custom validity, numeric stepping and text selection replacement.
Validation eligibility uses canonical control type, disabled/readonly attributes,
HTML datalist ancestry and Safari's fieldset/first-legend rules. Image inputs are
excluded, and readonly excludes every input type. The nearest disabled fieldset's
first legend exempts its controls even inside an outer disabled fieldset. Private
ancestor-disabled state preserves Safari's mutation behavior: some exemptions
survive fieldset attribute changes and removal, then refresh on insertion. The
shared state also governs script clicks, selection and form-data exclusion.
Shared tests cover 3,360 eligibility cases and 256 mutation sequences, including
shadow boundaries, namespaces, clone/adoption, descriptors and public overrides.
This eligibility contract does not assert complete constraint flags, localized
validation messages or validation UI.
Input/button setCustomValidity requires a DOMString argument, preserves UTF-16,
rejects Symbols and failed conversions without changing the previous error, and
emits no input/change/invalid events. Custom errors survive reset and adoption;
cloning/import starts with no custom error. Button ValidityState is stable and
live, with only customError affecting valid. Its message is empty while barred
from validation. Button checkValidity uses canonical private state and dispatches
a trusted, non-bubbling, cancelable invalid event; clearing the error or canceling
that event does not change the already-computed false return. Shared tests cover
595 Safari conversion/event cases plus descriptors, overrides and state lifetime.
The input constraint layer reads canonical values and attributes for required
text/date controls, checkbox/file requirements and named radio groups, including
detached groups. It checks email lists item by item, parses absolute URLs, compiles
Unicode-v patterns, and reuses finite decimal stepping for numeric/temporal bounds
and step mismatch, including reversed time ranges. Shared tests cover 3,262
scripted constraint cases and 1,260 radio-group cases. Native text/number edits
retain a private visible buffer and user-edit state, so incomplete numeric text
sets badInput while exposing an empty value, and UTF-16 length constraints apply
to user edits. Script assignments, type changes, reset, cloning and adoption
follow Safari's distinct state rules. Keyboard editing also handles numeric
filtering, maxlength, cancellation and internal selection for email/number
controls without exposing unsupported public selection APIs. Shared tests cover
85 typing sequences, 42 state mutations and 22 continued-typing sequences, plus
events, selection, numeric setter/stepping interactions and Unicode length limits.
These are JavaScript input operations; native date pickers and visual validation
UI are not supplied by this contract.
Input validationMessage uses the installed Safari English messages, including
error priority, normalized pattern titles, singular length messages and finite
decimal/date/time bound formatting. It is readonly and reads canonical state
without events, mutations or focus changes. Input checkValidity checks eligibility
and dispatches a trusted, cancelable, non-bubbling invalid event; cancellation or
listener mutations do not change its pre-event result. Safari's reversed time
ranges can have a nonempty message while checkValidity succeeds. Clearing a
populated temporal editor can leave its check result invalid until a subsequent
value, constraint, custom-error or eligibility update, although its live flags
and message are already clear. Private state preserves that distinction across
parser initialization, cloning, focus, reset and DOM moves. Shared tests cover
3,712 scripted message/event cases, 85 keyboard sequences and 792 cache transitions.
Rendering-driven picker updates and localized UI outside the measured English
profile are not part of these core JavaScript operations.
Input/button reportValidity preserves the core boolean result and trusted invalid
events, including canceled events and reentrant listener mutations. Its visual
presentation is an explicit adapted legacy no-op: no popup, scrolling or focus
change. The callable is non-constructible with Safari descriptors and receiver
checks. Shared tests verify 252 Safari return/event cases; separate runtime tests
verify the deliberate no-UI behavior, which differs from Safari focusing an invalid
connected control.
Input files accepts null/undefined as no-ops and requires a genuinely branded
FileList for other assignments, even when the input is not a file control. File
inputs retain the assigned list identity, including live DataTransfer changes.
Clearing the value or resetting the form empties that shared list; changing away
from file type discards the association, and cloning starts with an empty list.
Canonical file data drives the filename and required validity independently of
public property overrides. Assignment emits no input/change events or mutations.
Input src resolves against the owning document's live base URL and stores a
USVString attribute. Width/height use Safari unsigned conversion and return zero
outside image type. Image dimensions use attributes; CSS layout and intrinsic
image decoding remain explicit no-layout placeholders. These common input
surfaces stay in core, with 78 file-assignment, 308 URL and 360 dimension reference
cases plus shared state-lifetime tests.
Input string and boolean reflections use canonical null-namespace attributes,
WebIDL conversions and mutation records. Element receiver checks reject forged
prototype-only objects before entering native DOM operations. Input autocomplete
parses Safari's field, section, address/contact and webauthn tokens, including
hidden-input rules and live type changes. The shared corpus covers 1,068 cases;
it supplies metadata without an autofill or credentials backend. The legacy
`incremental` IDL property remains absent, as in installed Safari.
Input `maxLength`, `minLength` and `size` use canonical integer reflection with
Safari conversion and range errors; length setters check the opposite bound after
conversion. The 624-case corpus includes errors, attribute parsing and type changes.
`formMethod` includes `dialog`; method/encoding overrides retain missing and invalid
states. `formAction` resolves against its owning document and first eligible HTML
base, with 308 measured URL cases plus cloning/import/adoption checks. Created
documents retain Safari's empty internal action URL despite exposing `about:blank`
as `Document.URL`. These metadata contracts do not implement submitter routing,
POST transport or dialog submission; existing form submission supports GET.
Buttons share these form overrides, with a separate 308-case Safari URL reference.
Their name/value/formTarget strings and disabled/formNoValidate booleans preserve
canonical attributes, Web IDL conversion, mutation records and clone/import state.
The value attribute is independent of the button label. Missing/invalid button
types follow Safari's auto state: command/commandfor presence or a direct select
parent selects button behavior; otherwise the default is submit. Explicit submit,
reset and button types take precedence. Submission eligibility and native input
activation use canonical type state even when script shadows the public property.
Form named-property access (such as a button named reset shadowing form.reset)
remains a separate known gap; these contracts cover attribute reflection.
Native attribute values preserve every UTF-16 code unit, including lone surrogates
and NUL, across reflection, Attr objects, cloning/import/adoption, mutation records,
custom-element reactions and HTML serialization. ID/name/class lookups retain
those code units; URL parsing and CSS selector processing perform scalar conversion.
Failed URL reads preserve the original attribute string, as measured in Safari.
Text creation, textContent and character data use the same lossless native storage,
including cloning/import/adoption, mutation old values and serialization. Shared
Safari/Brimp tests round-trip every UTF-16 code unit. HTML parsing and inline script
evaluation retain separate scalar-string boundaries.
Anchor string attributes reflect canonical attributes; ping's setter converts lone
surrogates to replacement characters while its getter preserves raw attribute data,
as measured in Safari. referrerPolicy accepts Safari's keywords without trimming
or legacy aliases. Anchor text reads descendant text and replaces children through
the canonical DOM. These are core metadata/text contracts, without download or
ping networking behavior. hrefTranslate remains absent, matching installed Safari.
Anchor URL getters, component setters and stringification use canonical attributes
and the owning document's live base, independently of overridden public properties.
Missing/invalid links, opaque paths, file origins and partial/rejected host writes
follow installed Safari, including whether setters emit attribute mutation records.
The shared corpus covers 308 reads and 4,568 writes. Raw href attributes remain
unchanged by reads; query resolution follows document encoding, while assigning
search uses UTF-8. Real UTF-8, Windows-1252, Big5 and ISO-2022-JP documents verify
encoding, cloning and adoption. These URL operations do not navigate the page.
Token lists use private brands and canonical attributes. Their ordered-set methods,
conversion/error ordering, mutation records, live indexed reads and generic Array
iteration match the shared Safari corpus (1,288 operation cases). Anchor relList
retains identity through adoption and forwards assignments through the observable
relList/value properties. Relation support metadata follows Safari for anchors,
areas and links; those token names do not imply a rendering or resource-loading
backend. Object freezing/non-extensibility on token lists is unsupported: Brimp
rejects it so later attribute mutations can safely change the indexed properties.
Safari's native exotic object permits this combination; a JavaScript proxy cannot.
Input/button/anchor prototype inheritance, identity and descriptors match Safari.
HTML constructor policies follow installed Safari across 70 implemented interfaces.
Built-in constructors reject direct and subclass construction, except for Safari's
HTMLOutputElement factory. Explicit construction through HTMLElement uses a
registered autonomous name. customElements.define and document.createElement ignore
customized-built-in options, including option getters. Autonomous elements retain
construction, upgrades and lifecycle callbacks; construction stacks are per definition.
Failed fresh creation returns an HTMLUnknownElement, while failed upgrades retain
the original node and any prototype change already made by HTMLElement construction.
Failed elements are not upgraded again. These checks do not establish full registry
or parser conformance.
Script async exposes local creation state: newly created HTML script elements
start true, parsed scripts follow their async attribute, and property assignment
clears the initial force-async state and reflects the boolean attribute. Adding
the attribute also clears that initial state. This does not change the existing
dynamic script loader or establish cloning/import parity for internal script flags.
Link disabled reflects its boolean attribute even before a stylesheet exists;
stylesheet loading and cascade behavior retain their existing limitations.
Legacy frame contentDocument/contentWindow are null placeholders without a
frameset browsing-context engine. Iframe contentDocument/contentWindow retain the
existing lightweight child-document model; getSVGDocument returns that document
only when its actual root is SVG, otherwise null. No XML navigation engine is added.
Basic HTML interfaces include native span identity through creation, parsing,
cloning and adoption. Title text reads direct text-node children, excludes nested
element text and comments, and replaces children after DOMString conversion.
Its values and clones preserve UTF-16 code units. Meta name/httpEquiv/content/media/
scheme, div align, html version, list-item type/value and frameset rows/cols use
canonical attributes; shared Safari tests cover 266 conversion and mutation cases,
including signed-long list-item values. These reflections do not implement layout,
metadata-driven browser policies or frame rendering.
Body colors and frame/iframe margin strings use Safari's null-to-empty conversion
and canonical attributes. Embedded-element string, Boolean and keyword reflections
include iframe allow/loading/srcdoc and image sizes/loading/srcset. Image srcset
assignment converts lone surrogates to replacement characters; direct attribute
reads preserve raw UTF-16. The shared corpus covers 1,001 conversion and mutation
cases, with descriptor, receiver, namespace, cloning and adoption checks.
Iframe sandbox exposes a stable, live DOMTokenList with Safari token support
metadata and value forwarding. These attributes do not add permission or sandbox
policy enforcement, lazy loading, image resource selection or rendering. The
existing token-list freezing/non-extensibility limitation also applies to sandbox.
Safari-absent iframe allowPaymentRequest, csp and featurePolicy remain absent.
Image width/height/hspace/vspace use canonical numeric attributes for unrendered
elements; shared tests cover 240 Safari conversion and mutation cases. The Image
factory requires new, shares HTMLImageElement.prototype, respects subclass
prototypes and converts optional dimensions before reading newTarget.prototype.
Its 74-case corpus also verifies overflow and missing/undefined arguments.
Image resource and layout behavior uses explicit core legacy placeholders:
complete is always true, currentSrc is empty, intrinsic dimensions and x/y are
zero, and decode is a callable no-op returning undefined. Safari's decode returns
a Promise; the legacy naturalHeight empty-string default is corrected to numeric
zero. No image fetching, decoding, resource load/error events, CSS sizing or
intrinsic sizing is performed by these placeholders. They need no graphics plugin.
Embedded src/longDesc/lowsrc getters share anchor URL resolution and query encoding,
using the owning document's encoding metadata. Body background remains a raw
DOMString reflection, as in Safari. Shared checks cover 2,592 URL cases, 112
conversion cases, live bases, invalid URLs, canonical getters and adoption. Encoding
fixtures use matching HTTP and meta declarations for UTF-8, Windows-1252, Big5 and
ISO-2022-JP. Separate document-encoding tests cover 106 byte fixtures: BOMs take
precedence over HTTP charsets, then tokenized meta declarations, with Windows-1252
as the measured Safari default for unlabelled HTML. Existing encoding_rs decoders
convert HTML bytes before DOM parsing and inline script execution. The Lexbor tokenizer
ignores markup-looking strings in raw-text elements and comments; declarations
in the head can occur beyond the initial 1,024 bytes. No second HTML parser is used.
Document characterSet, charset and inputEncoding are readonly canonical aliases;
cloning and document.open/write retain the selected encoding, independently of
meta mutations and public-property overrides. DOMParser, new Document,
createHTMLDocument and template owner documents use UTF-8. Encoding is available
to plugins and init scripts before parsing. Lightweight iframe data navigation
shares the decoder, while srcdoc is parsed directly as Unicode with UTF-8 metadata.
This covers declared HTML encodings and the measured default, not XML decoding,
locale-dependent encoding detection, or full iframe browsing-context behavior.
Document URL/documentURI and contentType use canonical native metadata. The active
document follows history updates; DOMParser documents and clones retain URL
snapshots. Content types are normalized MIME essences from the response or document
creation path, available before plugins and init scripts. Public getters and
page-created properties cannot change aliases, clones or internal URL resolution.
Redirects inherit an omitted fragment and preserve an explicitly empty fragment,
including across multiple hops. Opening an attached iframe document takes the
caller's URL without its fragment; detached documents keep their URL. XML and
inert template documents reject document.open before mutation. Shared Safari
checks cover 12 navigation/lifecycle fixtures and four data/srcdoc iframe cases,
plus descriptors, receiver brands, adoption and public-property overrides.
Non-HTML response MIME metadata is exposed without adding text/JSON viewers or
XML parsing; these fields do not imply full browser document or frame support.
Document designMode retains on/off state per document (case-insensitive, ignoring
other values) and feeds isContentEditable, without an interactive editor.
activeElement falls back to that document's body/root when the focused node is
detached or belongs to another document. rootElement returns an SVG root or null.
scrollingElement uses the standards root/quirks body fallback without checking
CSS scrollability. HTMLDocument remains an alias of Document.
Document browser-service placeholders expose inactive fullscreen/picture-in-picture/
pointer-lock elements and saved Safari capability flags; they do not enable those
features. fonts, fragmentDirective, timeline, all and lastModified retain legacy
undefined defaults. In particular, all is not an HTMLDDA collection and lastModified
does not invent a timestamp. referrer is empty; XML metadata defaults to null/false/
null without XML declaration state. hidden/visibilityState/hasFocus retain fixed
visible/focused defaults. createProcessingInstruction is a callable no-op;
createCDATASection rejects with NotSupportedError (correct for HTML, also the
runtime's explicit limitation for non-HTML documents). CDATASection exposes its
interface only. Existing DOM creation, traversal, XPath, ranges, stylesheet state
and document writing continue to use their shared implementations.
Document title uses the first HTML title in document order, reading direct text
children and normalizing ASCII whitespace. An SVG root instead uses its first
direct SVG title and descendant text. Writes convert to DOMString before selecting
the target, update existing eligible titles, or create one in a direct HTML head
or as the first child of an SVG root. Other roots do not gain titles. Mutations
update observers and ranges, including identical nonempty writes. Document dir
reflects the null-namespace dir attribute only on an HTML html root, preserving
the assigned string while returning recognized values in lowercase. Both fields
use canonical DOM state and Safari descriptors, including on script-created
documents; no browser UI title or text layout is implemented.
Document documentElement is the first element child. Head selects the first direct
HTML head under that element; body requires an HTML html root and selects its
first direct HTML body or frameset. Body assignment accepts native HTML body and
frameset elements, replaces the selected body or appends under the root, and
preserves Safari's type/hierarchy errors, identity no-op, adoption, observer
records, range updates and custom-element lifecycle reactions. A non-HTML root
can receive an assigned body while the body getter remains null. These are DOM
tree operations; they do not implement frameset rendering or navigation.
Label control lookup and live labels lists use canonical attributes and tree state.
They follow measured Safari behavior for duplicate IDs, hidden controls, shadow
boundaries and form-associated custom definitions. Ordinary detached trees return
empty labels lists; detached shadow trees retain implicit labels. Lists preserve
Safari's ID-change cache behavior and invalidate on element-tree or changed
for/type attributes. htmlFor is a DOMString reflection with lossless UTF-16 storage.
Label click forwarding is a separate behavior. Form ownership includes parser-
assigned non-ancestor forms in malformed table markup. The native DOM preserves
these associations when both nodes move together and resets them when separated,
on form-attribute changes or cloning. Full document parsing and document.write
retain parser state across script/chunk boundaries; fragment parsing does not
create it. Input/button form and label.form are readonly canonical getters.
Shared Safari tests cover 245 parser association cases and 17 radio sequences,
plus moves, adoption, cloning, namespaces, exact IDs and document replacement.
Input datalist association follows Safari's supported types, first-ID lookup and
tree boundaries, including its exclusion of ordinary DocumentFragment roots.
`webkitdirectory` reflects boolean attribute presence. `webkitEntries` is a
stable frozen empty array per input; directory picking and filesystem entry
objects are not implemented. Safari's unsupported month/week types reflect text.
Input values distinguish attribute-backed button/hidden and checkbox/radio modes
from editable text and file modes. Text, email and URL sanitization, canonical
`defaultValue` reflection and dirty-value transitions follow measured Safari
behavior. Leaving file mode discards selected files while preserving the previous
edited value. Attribute and Attr-node changes use the same transition logic.
Date/time strings enforce Safari's calendar and time limits; datetime-local values
are normalized. Range values clamp and snap using Safari decimal behavior, with
canonical `min`/`max`/`step` reflections. The shared sanitization tests cover 743
measured cases through four mutation paths each. Decimal arithmetic is a private
adaptation of WebKit's BSD-licensed Decimal implementation; it adds no picker or
rendering backend.
Input valueAsNumber supports number, range, date, time and datetime-local. Date/time
conversion uses UTC components, Safari millisecond rounding and the existing
WebKit decimal helpers. Time serialization reflects step precision; datetime-local
serialization removes redundant zero fields. valueAsDate returns fresh Date objects
for date/time controls and accepts Safari's legacy Date, primitive-number and
primitive-string inputs. Invalid date serialization restores attribute-backed
values; NaN/infinite valueAsNumber assignments throw NotSupportedError without
changing state. Shared tests cover 2,340 Safari conversion, formatting and getter
cases, plus descriptors, coercion, intrinsic methods, cloning/reset and mutations.
Month/week remain Safari's text fallback. No picker or rendering backend is added.
Input stepUp/stepDown support number, range, date, time and datetime-local in core.
They use canonical input state, decimal step alignment and bounds, and Safari's
signed 32-bit count conversion. Temporal steps round to whole days or milliseconds
as appropriate. Stepping updates dirty values without input/change events or
attribute mutations. Shared tests cover 2,500 Safari cases, including decimal
rounding quirks, count overflow, unsupported types, invalid/any steps, default
restoration, calendar rollover and saturation, plus descriptors and clone/reset.
Color inputs parse named colors, CSS color functions, relative colors and color
mixing into sRGB or Display-P3 values using bundled CSSTools primitives and
Safari float conversion/rounding. The canonical `alpha` and `colorSpace`
reflections select serialization and preserve dirty-value semantics through
attribute changes, type transitions, reset and cloning. Shared tests compare 576
color cases through four paths and ten transition sequences with installed Safari.
System colors use measured Safari light-appearance defaults; they do not track
the host OS theme. Color parsing belongs to core and requires no graphics plugin.
Cloning and importing preserve input/textarea dirty values and input checkedness
and indeterminate state, including controls in template contents. Clones start
with a collapsed selection at zero and omit selected files, custom validity and
listeners. Textarea defaults use direct child text; values normalize CR/CRLF.
Input checkedness and defaultChecked use independent dirty state. Radio exclusivity
follows Safari's names, form ownership, tree boundaries, insertion and attribute
changes; cloning retains dirty state and reset restores defaults. Detached ordinary
trees, shadow trees and parsed documents follow measured Safari distinctions.
Checkbox/radio activation runs before click listeners and restores the appropriate
state on cancellation, shared by the input controller and script-dispatched
mouse/pointer clicks. Checkbox activation also clears indeterminate state.
Connected controls emit input/change events with Safari's shadow-boundary behavior.
Shared tests cover 556 recorded state, activation and group sequences, plus
canonical attributes, descriptors, cloning and reset in documents with/without a
browsing context. This does not add browser rendering or radio keyboard navigation.
Inputs and textareas share canonical UTF-16 selection state and range replacement.
The selection APIs follow measured Safari conversion, type gating, focus and
value/reset/type-transition behavior, including queued select events and coalesced
selectionchange events. The shared tests cover 640 selection/replacement cases.
Safari's case-insensitive setRangeText modes and preserve behavior for unknown
modes are retained. Selection painting and document Selection integration are
separate surfaces.
Required, email/URL, pattern and number/range constraints are checked. Full input
state algorithms, date/time stepping and user-edit length validation are
not yet covered. Reset clears dirty values/checkedness and honors cancellation.
Autonomous custom elements support creation and upgrade. ElementInternals supplies default
ARIA values, custom states and shadow-root access; form-associated custom element
submission/validation callbacks and an accessibility tree are not implemented.

Explicit document.open/write/writeln/close uses an incremental Lexbor parser;
retained nodes survive later writes, and inline classic scripts may write reentrantly.
Writing into an active navigation parser and external scripts supplied by
document.write are unsupported. These limits do not affect normal navigation's
external script loader. XML document parsing remains outside this layer.

FileReader supports asynchronous text, ArrayBuffer, binary-string and data-URL
reads, progress/lifecycle events, abort and reentrant reads. Data-URL reads encode
actual Blob bytes; Canvas mock output remains explicitly separate.

Web Streams use pinned `web-streams-polyfill`. The library owns readers/writers,
closure, errors, locking, piping,
backpressure and BYOB behavior. Fetch's byte stream releases native buffer credit
when a reader takes a chunk; unread network bytes stay bounded by the 256 KiB
queue. Response cloning tees the stream, so an unread clone branch can accumulate
buffered data as with the Streams tee algorithm. This is not a claim of complete
Fetch/body conformance.

## CSS and SVG

Core CSS support covers inline declarations and non-rendering stylesheets parsed
by Lexbor. CSSStyleSheet provides live cssRules, insertRule/deleteRule,
replace/replaceSync, and rule ownership. Style rules and media/supports groups
support editing and serialization; declaration values use the same Lexbor-backed
CSSStyleDeclaration as inline styles. Style elements own sheets; link elements
load sheets through Fetch and restrict cross-origin rule access. Document and
ShadowRoot expose styleSheets and mutable adoptedStyleSheets collections, with
constructed-sheet and owner-document checks.

This is not complete CSS grammar coverage: @import loading, namespace rule editing,
keyframes, layers, full font-face descriptors and media-condition normalization are
not implemented. Unsupported rules/declarations can be discarded by the parser;
stylesheets do not participate in a cascade. The optional Typed OM plugin provides
inline value objects and attributeStyleMap operations.

Cascade, computed style, style-dependent geometry, animation timelines and rendering
are unsupported. Generic SVG elements participate in the DOM; specialized SVG
interfaces, XML parsing and XMLSerializer remain unsupported.

## Structured cloning

`structuredClone` uses `core-js-pure` with the engine's ArrayBuffer transfer. It
preserves cycles and shared references in supported JavaScript values, including
Maps/Sets, dates and binary views. Functions, DOM nodes, CryptoKeys and unsupported
platform objects reject with DataCloneError. Message channels use this shared
clone implementation too.

The [compatibility corpus](../compatibility/README.md) records exact upstream
artifacts and Chromium comparisons. Detector findings, including CreepJS, remain
observational and do not define the product's API boundary.
