# Browser primitives

Pinned, bundled runtime dependencies. Rebuild with `npm ci --ignore-scripts && npm run build` in this directory. Cargo and binary package consumers use the checked-in bundles and do not need npm or network access. Lockfile integrity, generated bundle hashes and license files are retained here. Do not edit generated bundles.

- `core-js-pure` supplies structured cloning of supported JavaScript data types, cycles, Maps/Sets, dates, buffers/views and ArrayBuffer transfer. It rejects unsupported platform types with DataCloneError; it does not pretend to clone DOM nodes or CryptoKeys.
- `web-streams-polyfill` supplies WHATWG-based readable/writable/transform and byte streams, including backpressure, cancellation and reader/writer lifecycle. Fetch attaches its native source to these streams.
- `xpath` supplies the XPath 1.0 expression engine over Brimp's live DOM wrappers, with an HTML namespace adapter. This is not a claim of full browser XPath conformance (in particular, iterator invalidation on later DOM mutation is not supplied by this library).
- `fake-indexeddb` replaces the handwritten IndexedDB subset in the optional storage plugin. Transactions, indexes, rollback and structured-cloned records are provided by the library. The factory is instantiated in each realm. Navigation discards its data; it does not provide shared origin databases or durable storage.

`core.js` and `storage*.js` are the authored adapters. The core bundle installs no storage or graphics capabilities. Storage is still an explicitly selected plugin.

The core build replaces core-js-pure's `internals/shared-store.js` with a
bundle-local CommonJS store. All internal consumers still share that one store;
it never creates `globalThis.__core-js_shared__`. Dependency versions/licenses
remain in the host-side manifest and notices. This build adapter is specific to
our single bundled instance and does not alter core-js loaded by page code.
