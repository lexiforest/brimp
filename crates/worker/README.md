# lite-worker

The `lite-worker` package contains the worker executable and its Chrome DevTools Protocol
subset. It reads length-prefixed CDP JSON from a local stream inherited from the
Brimp daemon. The daemon owns the public HTTP discovery and WebSocket boundary.

See `SUPPORT.md` for the exact tested method subset.

`serve_framed` accepts any Tokio asynchronous byte stream. The
`lite-worker` executable supplies its inherited Unix socket; a Windows
host can supply a named pipe without changing protocol dispatch.
