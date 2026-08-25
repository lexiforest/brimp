# Brimp CDP

`brimp-cdp` exposes a deliberately small Chrome DevTools Protocol server over
HTTP discovery and WebSocket transport. It is a remote server/client boundary;
unlike the Python and Node native extensions, protocol values are serialized as
JSON and screenshots are base64 encoded.

The server binds to `127.0.0.1:9222` by default. A non-loopback bind is rejected
unless `--allow-non-loopback` is passed, in which case the server prints a
security warning before binding.

See `SUPPORT.md` for the exact tested method subset.

Run `./crates/cdp/puppeteer-test.sh` to install the exact locked Puppeteer
version into a temporary directory and execute the recorded workflow.
