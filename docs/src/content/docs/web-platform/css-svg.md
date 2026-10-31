---
title: CSS, SVG and geometry
description: Edit styles and SVG DOM without a cascade, layout or rendering engine.
---

Brimp can parse and edit styles, but it does not apply a CSS cascade or render
HTML/SVG. Use styles as document data; do not rely on them for element dimensions,
hit testing, visibility or computed values.

| Surface | Behavior |
| --- | --- |
| Inline declarations and stylesheets | Parsed values, rule editing, serialization and DOM ownership |
| Additional CSS property names | Some are local placeholders and do not serialize into CSS |
| Computed style and layout | Unavailable; dimensions and UI methods expose documented placeholders |
| SVG | DOM nodes, attributes and selected interface shapes; no SVG renderer |
| Core Typed OM and geometry | Value arithmetic and inline edits, not document measurements |

## Stylesheets and declarations

Inline styles and stylesheet declarations share the Lexbor parser. Unsupported
rules may be discarded. Stylesheets never participate in a cascade.

<details>
<summary>Detailed behavior and edge cases</summary>

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
stylesheets do not participate in a cascade. Core Typed OM provides
inline value objects and attributeStyleMap operations.

Cascade, computed style, style-dependent geometry, animation timelines and rendering
are unsupported. SVG elements participate in the DOM with the limited interfaces described below.
The full SVG interface hierarchy, XML parsing and XMLSerializer remain unsupported.

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
primitives. Numeric CSS factories and rectangle arithmetic are available in the
core Typed OM and geometry APIs, without document measurement or rendering.

</details>

## Geometry and UI methods

Geometry supports points, rectangles, quads and affine 2D matrices without layout.
Typed OM supports unit values and inline attributeStyleMap edits, without cascade
or general compound-unit algebra.

Dimensions are zero placeholders; scroll offsets can store local values without
scrolling. Animation, fullscreen, hit-testing and editing calls do not create UI
effects, promises or geometry results.

<details>
<summary>Detailed behavior and edge cases</summary>

Core Element and Document expose legacy callable placeholders for scrolling,
animation, hit testing, fullscreen, picture-in-picture exit, pointer capture/lock,
and document editing commands. They return undefined without changing DOM or UI
state, dispatching events, or providing promises, animations or geometry results.
The deprecated document captureEvents, releaseEvents and clear methods are also
no-ops. These contracts support scripts that discard the results; they do not
provide browser rendering or interactive editing. 

Element client/scroll dimensions and border offsets are zero placeholders.
scrollTop/scrollLeft retain assigned finite numbers locally, without scrolling,
clamping to layout bounds or emitting events. computedStyleMap returns undefined;
the core attributeStyleMap still edits real inline declarations.
HTMLElement.isContentEditable follows contenteditable ancestors and the document
designMode flag, without implementing an editor. nonce is ordinary attribute
reflection, without CSP nonce hiding. outerText reads the existing raw innerText
fallback and replaces its element with text/BR nodes when assigned; rendered-text
whitespace/layout semantics are not implemented.

</details>

## SVG

SVG nodes participate in the DOM and expose selected reflection and graphics
interfaces. Bounds/matrix methods return zero/identity value objects; animated
values have no animation engine.

<details>
<summary>Detailed behavior and edge cases</summary>

SVG elements share nonce, autofocus and tabIndex reflection, and expose their
nearest SVG ancestor through ownerSVGElement/viewportElement. className is a
stable baseVal/animVal facade backed by the class attribute; animVal mirrors the
base value without an animation engine. The `svg` root uses SVGSVGElement,
shape elements (circle, ellipse, rect, path, line, polyline, polygon) use
SVGGeometryElement, other listed graphics elements use SVGGraphicsElement and the
rest retain generic SVGElement wrappers. getBBox returns a zero DOMRect and
getCTM/getScreenCTM return identity DOMMatrix values, while transform,
requiredExtensions and systemLanguage retain legacy empty-string placeholders.
SVGSVGElement adds createSVGPoint/createSVGMatrix/createSVGRect and
SVGGeometryElement adds isPointInFill/isPointInStroke (false), getTotalLength (0)
and getPointAtLength (the origin). There is no SVG renderer, matrix computation,
animated transform list or full SVG interface hierarchy. DOM attributes, styles,
events and cloning continue to work.

</details>


## Verification

The [runtime tests](https://github.com/lexiforest/brimp/tree/main/tests/runtime/dom/)
record the behavior behind this reference. Safari comparisons cover the recorded
cases; they do not establish complete browser conformance. The
[field ledger](https://github.com/lexiforest/brimp/blob/main/COVERAGE.yaml) tracks
individual decisions and evidence.
