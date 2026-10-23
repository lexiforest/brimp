# Lexbor provenance

- Upstream: https://github.com/lexbor/lexbor
- Release: **v3.0.0** (pinned; no build-time download)
- Source archive: https://codeload.github.com/lexbor/lexbor/tar.gz/refs/tags/v3.0.0
- Archive SHA-256: `eafaa79ef9871f0bbb1978eda8677d184f7ecdcaa203d7cd25b3f86e32c014c2`
- Imported paths: `source/lexbor/{core,dom,ns,tag,html,css,selectors,ports}`,
  `LICENSE`, `NOTICE`. Unused upstream modules are omitted. Source files are unmodified except the documented patches below.
- Built modules: core, dom, ns, tag, html, css, selectors.

The application boundary is in `../../native/lexbor.c`. It builds the canonical
Lexbor DOM, uses independent parser instances for nested fragment parsing, and
never frees individual DOM nodes while a document is alive. The arena is released
only when its owning document is dropped. No rendering/style engine is built.

Parser suspension feeds chunks ending at `>` (using `memchr`, not an HTML parser)
and observes Lexbor's completed script-end token after the tree builder consumes
it. A callback reports the corresponding canonical node before any following
chunk is supplied. Comments, raw text, quoted delimiters, script escapes, and
malformed markup are interpreted by Lexbor. A separate fragment parser allows
script-time `innerHTML` mutations without disturbing the suspended parser.

## Local selector patch

`source/lexbor/selectors/selectors.{c,h}` adds an explicit scoping-root pointer
and `source/lexbor/css/selectors/state.c` enables parsing of `:scope`. The
matcher implements the previously unimplemented `:scope` branch using that pointer.
The shim sets it to the query root (or document element for document queries).
Parsing, combinators, nested selectors, and matching remain Lexbor operations.
No source rewriting or alternative selector engine is used.

## Parser mutation callback patch

`source/lexbor/html/tree.h` uses the event-emitting insert-before operation for
foster-parented nodes, matching the existing event-emitting child insertion path.
The shim temporarily installs document mutation callbacks only while the parser
runs, restores them before scripts, and drains owned node records at boundaries.
The document mutation callback struct also has a character-data callback, invoked
before the HTML tree builder appends to an existing text node. This preserves the
old value without snapshotting or duplicating the DOM. Other consumers leave this
zero-initialized callback unset.

## Attribute namespace patch

`html/tree.c` and `dom/interfaces/element.c` assign the null namespace to plain
attributes; namespace adjustment still assigns XML/XMLNS/XLink namespaces.
The element's ID/class caches only track null-namespace attributes.
`dom/interfaces/attr.c` interns prefixes in the prefix table rather than the
namespace table. The shim exposes namespace-aware attribute mutation against
this canonical state, with no parallel attribute store.
