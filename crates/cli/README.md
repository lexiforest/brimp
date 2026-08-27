# brimp-cli

Thin command-line interface over `web-runtime`'s canonical automation page.

```text
brimp doctor
brimp cdp [--bind HOST:PORT] [--allow-non-loopback]
brimp eval URL --js EXPRESSION [--timeout-ms N]
brimp screenshot URL --output PATH [--full-page] [--overwrite] [--timeout-ms N]
```

Structured results are written to stdout, diagnostics to stderr, and PNG bytes
directly to the requested file. Existing files are refused unless
`--overwrite` is explicit. Exit categories are stable: 2 input, 10 transport,
11 HTTP status, 12 navigation, 13 JavaScript, 14 timeout, 15 cancellation, 16
unsupported result, 17 closed object, and 18 screenshot/runtime failure.

`fetch`, `extract`, `render`, and `batch` are intentionally absent until their
canonical core semantics and bounded batch isolation are implemented.

See `SUPPORT.md` for the exhaustive tested command surface.
