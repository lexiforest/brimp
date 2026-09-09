# brimp-web-apis

Manual JavaScriptCore bindings for Brimp's browser-facing APIs.

The binding runtime maps stable JavaScript wrappers to Blitz node identifiers
and implements the initial DOM, CSSOM, event, timer, fetch, cookie, `Location`,
`URL`, `URLSearchParams`, and `Navigator` surfaces. Every DOM and layout
operation reads or mutates the canonical `BrowserDocument` directly.

Key internal components include:

- `BindingRuntime`, which installs and resets page globals and prototypes;
- `WrapperCache`, which preserves JavaScript object identity for native nodes;
- `TimerQueue` and `FetchQueue`, which hand work back to the page owner thread;
- `BrowsingContext`, which stores URL and cookie state.

The core browser JavaScript is split by API domain under `src/runtime/` and
concatenated into one dependency-ordered evaluation unit. Optional Canvas,
worker, streaming-networking, and persistent-storage scripts are installed
independently according to page options. Native dispatch is split along the
same subsystem boundaries.

This crate is primarily an implementation layer for `brimp-runtime`. Applications
should normally construct a `brimp_runtime::Browser` and interact with a `Page`
instead of installing bindings directly.
