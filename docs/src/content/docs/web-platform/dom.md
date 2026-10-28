---
title: DOM and documents
description: Parse, query and edit HTML, with explicit document and collection limits.
---

Use the DOM to inspect and edit HTML without rendering it. Nodes retain their
identity across edits and detachment; page navigation creates a new realm and
invalidates host handles from the old one.

| Available | Main limits |
| --- | --- |
| Selectors, traversal, mutation, attributes, collections and ranges | Range geometry and rendered-text layout are unavailable |
| HTML parsing, document writing and declared character encodings | No XML parsing, XMLSerializer, or general iframe browsing-context engine |
| Shadow DOM and autonomous custom elements | No declarative shadow roots, manual slot assignment or full focus delegation |

The sections below describe page-side JavaScript. For host handle ownership, see
[Values and handles](/guides/values/).

## Element geometry

There is no layout engine, so `getBoundingClientRect()` returns a zero `DOMRect`
at the origin and `getClientRects()` returns an empty `DOMRectList` rather than
`undefined`. The result objects are real value objects, so scripts that read
`.width`, `.top`, `.right` or `rects.length` keep working; nothing is measured,
and unrelated receivers raise `TypeError` like other DOM methods.

## Parse and create documents

HTML is parsed into one native DOM. Lifecycle events reflect script execution.
`DOMParser` and explicit document writing are supported within the limits below.

<details>
<summary>Detailed behavior and edge cases</summary>

The browser corpus exercises input handlers/activation, token lists,
NodeIterator, attribute removal, XPath and non-rendering Shadow DOM. Shadow roots
have isolated canonical subtrees, named slots, connected roots and composed event
retargeting, including closed-root path filtering. Custom element definitions
read observedAttributes before construction, as required by Lit's initialization.
There is no CSS cascade, computed geometry or rendering. Manual slot assignment,
declarative shadow roots and full focus delegation remain
outside this implementation. XPath uses the pinned `xpath` library; mutation
invalidation of XPath iterators is not yet implemented.

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

`document.doctype` returns the canonical DocumentType node or null. DocumentType
exposes readonly name/publicId/systemId, cloning, adoption and ChildNode methods.
Importing a DocumentType throws NotSupportedError, matching installed Safari.
Document compatibility mode comes from the parser and is retained when cloning;
removing the doctype does not change it. DOMParser mode selection follows
installed Safari's fast fragment eligibility, including supported tags/nesting,
attribute syntax, and text/depth limits. Lexbor continues to build the DOM.
The shared mode tests compare 606 inputs with measured Safari results; this
does not claim complete browser parsing conformance.

Explicit document.open/write/writeln/close uses an incremental Lexbor parser;
retained nodes survive later writes, and inline classic scripts may write reentrantly.
Writing into an active navigation parser and external scripts supplied by
document.write are unsupported. These limits do not affect normal navigation's
external script loader. XML document parsing remains outside this layer.

</details>

## Read and change the tree

Selectors and tree edits use live native state. Moving a node preserves its
identity; cloning copies supported DOM state without copying listeners or expandos.

<details>
<summary>Detailed behavior and edge cases</summary>

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

</details>

## Collections and token lists

`children` and `childNodes` are live; `querySelectorAll()` returns a snapshot.
Live collections and token lists cannot be frozen or made non-extensible.

<details>
<summary>Detailed behavior and edge cases</summary>

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

NodeList is a non-Array collection with readonly indexed entries and length;
scripts cannot construct it directly. childNodes and other live producers update
after DOM mutations, while querySelectorAll and mutation records retain snapshots.
item converts an unsigned index and returns null outside the list. entries, keys,
values, forEach and Symbol.iterator use generic Array functions as in Safari.
Freezing and preventing extensions are unsupported, matching the explicit limit
of Brimp's other proxy-backed live collections. Four representative shared tests
cover ordinary reads, enumeration, mutation, snapshots and iteration.

Token lists use private brands and canonical attributes. Their ordered-set methods,
conversion/error ordering, mutation records, live indexed reads and generic Array
iteration match the shared Safari corpus (1,288 operation cases). Anchor relList
retains identity through adoption and forwards assignments through the observable
relList/value properties. Relation support metadata follows Safari for anchors,
areas and links; those token names do not imply a rendering or resource-loading
backend. Object freezing/non-extensibility on token lists is unsupported: Brimp
rejects it so later attribute mutations can safely change the indexed properties.
Safari's native exotic object permits this combination; a JavaScript proxy cannot.

</details>

## Text and attributes

Text and attributes preserve UTF-16 code units, including lone surrogates.
Text edits update ranges and mutation observers; URL and selector processing
have separate string-conversion rules.

<details>
<summary>Detailed behavior and edge cases</summary>

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

Native attribute values preserve every UTF-16 code unit, including lone surrogates
and NUL, across reflection, Attr objects, cloning/import/adoption, mutation records,
custom-element reactions and HTML serialization. ID/name/class lookups retain
those code units; URL parsing and CSS selector processing perform scalar conversion.
Failed URL reads preserve the original attribute string, as measured in Safari.
Text creation, textContent and character data use the same lossless native storage,
including cloning/import/adoption, mutation old values and serialization. Shared
Safari/Brimp tests round-trip every UTF-16 code unit. HTML parsing and inline script
evaluation retain separate scalar-string boundaries.

</details>

## URLs and character encodings

URL-valued attributes follow the owning document’s base and encoding. HTML byte
decoding uses BOM, HTTP charset and meta declarations, in that order; unlabelled
HTML uses the measured Safari default, Windows-1252.

<details>
<summary>Detailed behavior and edge cases</summary>

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

</details>

## Document structure and metadata

Document title, direction, head/body selection and editing flags reflect DOM
state. Visibility and browser-service fields include fixed values; they do not
create UI or an editor.

<details>
<summary>Detailed behavior and edge cases</summary>

Document designMode retains on/off state per document (case-insensitive, ignoring
other values) and feeds isContentEditable, without an interactive editor.
activeElement falls back to that document's body/root when the focused node is
detached or belongs to another document. rootElement returns an SVG root or null.
scrollingElement uses the standards root/quirks body fallback without checking
CSS scrollability. HTMLDocument remains an alias of Document.
Document browser-service placeholders expose inactive fullscreen/picture-in-picture/
pointer-lock elements and saved Safari capability flags; they do not enable those
features. fragmentDirective and all retain legacy undefined defaults. timeline
returns a stable per-document DocumentTimeline whose currentTime follows the realm
clock, duration is null and getAnimations() is empty; lastModified formats the
document load time once per document as `MM/DD/YYYY HH:MM:SS` in local time and
does not read a response Last-Modified header. FontFace/FontFaceSet metadata is described under
[Window and environment](/web-platform/environment/#fonts). In particular, all is not an HTMLDDA collection. referrer is empty; XML metadata defaults to null/false/
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

</details>

## Custom elements

Autonomous custom elements support construction, upgrades and lifecycle
callbacks. Customized built-ins and form-associated submission/validation
callbacks are not implemented.

<details>
<summary>Detailed behavior and edge cases</summary>

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

Autonomous custom elements support creation and upgrade. ElementInternals supplies default
ARIA values, custom states and shadow-root access; form-associated custom element
submission/validation callbacks and an accessibility tree are not implemented.

</details>

## HTML elements, scripts and frames

Common HTML metadata reflects attributes. Script properties use the existing
loader; iframe documents use a lightweight child-document model. Frame attributes
and sandbox token metadata do not enforce a browser security policy.

<details>
<summary>Detailed behavior and edge cases</summary>

Link and script metadata use the common attribute reflection, URL resolution and
token-list machinery. Links expose imageSizes/imageSrcset for preload metadata;
scripts expose referrerPolicy and text. Script text reads direct text children,
and dynamic script execution uses that same source. These properties do not add
image preload processing or new script scheduling modes.

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

</details>

## Images

Image attributes and constructors work, but core image elements do not fetch or
decode resources. `complete` is always true, `currentSrc` is empty and intrinsic
dimensions are zero. `decode()` returns `undefined`, not a Promise.

<details>
<summary>Detailed behavior and edge cases</summary>

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

</details>


## Verification

The [runtime tests](https://github.com/lexiforest/brimp/tree/main/tests/runtime/dom/)
record the behavior behind this reference. Safari comparisons cover the recorded
cases; they do not establish complete browser conformance. The
[field ledger](https://github.com/lexiforest/brimp/blob/main/COVERAGE.yaml) tracks
individual decisions and evidence.
