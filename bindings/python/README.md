# Brimp Python binding

PyO3 extension plus an idiomatic asynchronous `brimp` package. `launch`, page
lifetime, navigation, structured evaluation, document output, PNG bytes, and
idempotent close delegate to `web-runtime`. Cancelling `Page.goto` sets the
core cancellation token and waits for request cleanup before propagating
`CancelledError`.

This is an in-process native extension, not a CDP client. See `SUPPORT.md` for
the exhaustive tested API and error-code surface.
