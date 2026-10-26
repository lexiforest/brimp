# Browser primitives

Pinned, bundled runtime dependencies. Rebuild with `npm ci --ignore-scripts && npm run build` in this directory. Cargo and binary package consumers use the checked-in bundles and do not need npm or network access. Lockfile integrity, generated bundle hashes and license files are retained here. Do not edit generated bundles.

- `core-js-pure` supplies structured cloning of supported JavaScript data types, cycles, Maps/Sets, dates, buffers/views and ArrayBuffer transfer. It rejects unsupported platform types with DataCloneError; it does not pretend to clone DOM nodes or CryptoKeys.
- `web-streams-polyfill` supplies WHATWG-based readable/writable/transform and byte streams, including backpressure, cancellation and reader/writer lifecycle. Fetch attaches its native source to these streams.
- `xpath` supplies the XPath 1.0 expression engine over Brimp's live DOM wrappers, with an HTML namespace adapter. This is not a claim of full browser XPath conformance (in particular, iterator invalidation on later DOM mutation is not supplied by this library).
- `fake-indexeddb` replaces the handwritten IndexedDB subset in the optional storage plugin. Transactions, indexes, rollback and structured-cloned records are provided by the library. The factory is instantiated in each realm. Navigation discards its data; it does not provide shared origin databases or durable storage.
- `@csstools/css-color-parser`, its tokenizer/parser and color helpers supply CSS color parsing and conversion for core input values. `input-color.js` adapts sRGB byte rounding, Display-P3 and alpha serialization, currentColor and measured Safari light-appearance system defaults. It does not render or open a color picker. The separate bundle lives inside the DOM runtime closure and exposes no page global.

The color build adapts the pinned library at a few verified boundaries:
`safari-color-math.js` supplies WebKit float arithmetic and its piecewise
Rec.2020 transfer (including Safari's signed-negative behavior);
`safari-color-conversion.js` avoids unnecessary XYZ round trips within the sRGB
family. Channel parsing preserves extended floats and infinities instead of
clamping them to signed integers. These adapters also apply inside relative
colors and color-mix. The dependency files on disk remain unchanged; the build
fails if its expected functions, channel limits or exports change.
The shared `input-color.test.mjs` corpus verifies 576 cases in both color spaces,
four value-setting paths, and ten attribute-transition sequences against installed
Safari. WebKit's BSD notice is retained with the adapted math and distributed
with the package licenses. See [WebKit's color transfer functions](https://github.com/WebKit/WebKit/blob/main/Source/WebCore/platform/graphics/ColorTransferFunctions.h)
and [CSS Color Parser](https://github.com/csstools/postcss-plugins/tree/main/packages/css-color-parser).

`core.js`, `input-color.js`, `safari-color-*.js` and `storage*.js` are the authored adapters. The core bundle installs no storage or graphics capabilities. Storage is still an explicitly selected plugin.

The core build replaces core-js-pure's `internals/shared-store.js` with a
bundle-local CommonJS store. All internal consumers still share that one store;
it never creates `globalThis.__core-js_shared__`. Dependency versions/licenses
remain in the host-side manifest and notices. This build adapter is specific to
our single bundled instance and does not alter core-js loaded by page code.
