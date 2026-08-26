# web-bindings

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

This crate is primarily an implementation layer for `web-runtime`. Applications
should normally construct a `web_runtime::Browser` and interact with a `Page`
instead of installing bindings directly.
