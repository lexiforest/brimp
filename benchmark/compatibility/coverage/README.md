# jsdom and happy-dom coverage audit

This is an initial source-guided audit of browser DOM behavior, not a claim of
full coverage of either project. The 53 probes deliberately target likely gaps;
the counts are not representative compatibility percentages.

## Reproduce

Clone the source repositories for inspection:

```sh
git clone --depth 1 https://github.com/jsdom/jsdom.git /tmp/brimp-coverage-jsdom
git clone --depth 1 https://github.com/capricorn86/happy-dom.git /tmp/brimp-coverage-happy-dom
```

Prepare the native binding and corpus dependencies as described in
[the corpus guide](../README.md), then run from the workspace root:

```sh
DYLD_FRAMEWORK_PATH="$PWD/../WebKit/WebKitBuild/Release" \
  node benchmark/compatibility/coverage/run.mjs
```

Set PLAYWRIGHT_BROWSERS_PATH if Chromium is installed in a custom directory.
JSDOM_SOURCE and HAPPY_DOM_SOURCE select alternative clone locations. The runner
records their commits, the pinned npm versions, native binding hash, probe/runner
hashes, package-lock hash, browser version and raw observations. No external
website is used. Browser/native navigation is fulfilled with local fixture HTML;
Node DOM windows are initialized from the same HTML and origin.

Each probe gets a fresh realm. Each engine runs in a separate process with a
90-second deadline. API results are copied as JSON; exception names/messages are
retained. Chromium is the behavioral reference. Failed references or incomplete
Brimp runs invalidate the affected comparisons. The current runner uses four configurations: Chromium, Brimp with its default
browser plugins, jsdom and happy-dom. Storage, geometry and CSS Typed OM are core
APIs. The saved report below predates those ownership and default-selection changes.

## Source and execution provenance

Reviewed checkouts:

- jsdom: `/tmp/brimp-coverage-jsdom`, commit
  `89adb92b23d5b0dd9bf8263181783bfde74b2f6d`.
- happy-dom: `/tmp/brimp-coverage-happy-dom`, commit
  `fd163520b74d9fa146be91f7b4e4f46d6cccbe13`.

Execution uses the corpus's **jsdom 30.1.3 and happy-dom 21.0.4 npm releases**.
The clones guide source/test inspection; they are not built or represented as
executed upstream suites. [sources.json](sources.json) identifies reviewed files,
commit-pinned source links, hashes, and whether the corresponding packaged source
file is identical. The probes are independently authored small scenarios.

Source selection includes jsdom's Document/NamedNodeMap/TreeWalker IDL and node,
form and custom-element implementation, its collection-liveness and Range tests,
and happy-dom's document, node, form, Range, stylesheet and XML tests. For example:
[jsdom collection tests](https://github.com/jsdom/jsdom/blob/89adb92b23d5b0dd9bf8263181783bfde74b2f6d/test/web-platform-tests/to-upstream/dom/collections/HTMLCollection-namedItem-liveness.html),
[jsdom TreeWalker interface](https://github.com/jsdom/jsdom/blob/89adb92b23d5b0dd9bf8263181783bfde74b2f6d/lib/jsdom/living/traversal/TreeWalker.webidl),
[happy-dom form tests](https://github.com/capricorn86/happy-dom/blob/fd163520b74d9fa146be91f7b4e4f46d6cccbe13/packages/happy-dom/test/form-data/FormData.test.ts),
[happy-dom stylesheet tests](https://github.com/capricorn86/happy-dom/blob/fd163520b74d9fa146be91f7b4e4f46d6cccbe13/packages/happy-dom/test/css/CSSStyleSheet.test.ts).

## Recorded coverage

The saved historical run completes all 53 probes in all five configurations. All Chromium
references succeed. Brimp core matches **45**, and the bundled configuration
matches **47**. jsdom matches 50 and happy-dom 36. This deliberately gap-focused
sample is not a representative compatibility percentage or a full upstream suite.

| Area | Passing behavior |
| --- | --- |
| Microtasks and observers | FIFO ordering with Promise callbacks; observer batching, old values and delivery at a Promise checkpoint. |
| Collections | Retained childNodes/children identity and liveness; named property enumeration; HTML-only name matching. |
| Replacement and Range | Replacement boundaries collapse at the removed node's index; invalid replacement throws before mutation. |
| Documents and attributes | Adoption/import, Attr attachment and namespace maps, namespace queries/lookup, inert template ownership, document writing. |
| Traversal | TreeWalker directional navigation and rejected-subtree pruning; NodeIterator removal recovery. |
| Forms | Required/custom validity, numeric stepping, selection replacement, external association, labels, reset and FormData cases. |
| Custom elements | Autonomous upgrade/whenDefined, customized built-ins and ElementInternals default ARIA. |
| CSSOM | Inline priority, style-owned rule editing, constructed sheets and adoption. |
| Other core behavior | Shadow slots, listener abort, :has, FileReader and partial Range extraction/text boundaries. |

That run recorded eight core mismatches:

| Probe IDs | Boundary |
| --- | --- |
| xml-parser; xml-serializer; xml-document-cdata | XML parsing/serialization and XML documents are unsupported. |
| svg-animated-length; svg-class-name | Specialized SVG interfaces and animated values are unsupported. |
| computed-style-cascade | Cascade and computed style are outside the current core. |
| storage-basic; geometry-matrix | Plugin APIs in that run; now installed in core. |

All eight had at least one peer matching Chromium. They do not authorize broader
scope. That revision corrected 28 of the initial 36 core mismatches in this probe
set. These counts are historical, not a measurement of the current checkout. Additional regression tests exercise failures and invariants beyond
the probes: replacement observer records, namespace isolation/case, retained
adopted template content, filter recursion, reset cancellation, stylesheet ownership
and cross-origin access, incremental/reentrant document writing, and native memory
lifetimes under sanitizers.

## Limits and next priorities

Passing a probe verifies its scenario, not every method on an interface. See
[core coverage](../../../docs/src/content/docs/web-platform/index.md) for the supported CSS rule types,
form/input limitations, ElementInternals boundary and document.write restrictions.
Full form-associated custom elements, complete CSS grammar, parser-inserted external
scripts through document.write and writing into a navigation parser need separate
workload evidence and implementation work.

The prototype inventory remains a list of candidates for behavioral tests, not a
missing-API total. XML and specialized SVG remain workload-driven decisions.
Storage, geometry and CSS Typed OM now live in core; computed style and rendering
are not implied by their presence or by this audit. Networking/security policy, full HTML and
Shadow DOM conformance, storage lifetimes, graphics, workers and host-only library
APIs are not covered by these 53 probes.

[results.md](results.md) has every comparison. [report.json](report.json) contains
exact outputs, errors, inventories and runtime provenance; [probes.mjs](probes.mjs)
contains the runnable cases. The static inventory is a queue for further testing,
not a complete missing-API checklist.
