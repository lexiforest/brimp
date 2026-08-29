---
title: CLI API
description: Commands, flags, output contracts, and exit codes for brimp.
---

The CLI writes structured results to standard output and diagnostics to
standard error.

```text
brimp doctor
brimp eval URL --js EXPRESSION [--persona PATH] [--timeout-ms N] [PAGE OPTIONS]
brimp screenshot URL --output PATH [--persona PATH] [--full-page] [--overwrite] [--timeout-ms N] [PAGE OPTIONS]
brimp cdp [--bind HOST:PORT] [--allow-non-loopback] [PAGE OPTIONS]
```

Run `brimp help` for the top-level synopsis. `brimp cdp --help` prints the CDP
server's page-option synopsis.

## `brimp doctor`

Validates JavaScriptCore, libcurl-impersonate, and the selected transport
profile. It creates and closes a page to verify the complete native runtime,
then writes one JSON object:

```json
{"javascriptCore":"ok","libcurlImpersonate":"ok","profile":"chrome150"}
```

The exact profile value follows the default persona.

## `brimp eval`

```text
brimp eval URL --js EXPRESSION [--persona PATH] [--timeout-ms N]
```

Navigates to `URL`, evaluates JavaScript, and prints one JSON value. The default
timeout is 30,000 milliseconds. Ctrl-C cancels navigation and its active network
request.

```sh
brimp eval https://example.com --js '({title: document.title, links: document.querySelectorAll("a").length})'
```

Strings are printed as JSON strings, not as unquoted terminal text. Values that
cannot be represented by JSON—such as functions, symbols, `BigInt`, cycles, or
top-level `undefined`—fail with exit code 16. A thrown JavaScript exception uses
exit code 13.

## `brimp screenshot`

```text
brimp screenshot URL --output PATH [--persona PATH] [--full-page] [--overwrite] [--timeout-ms N]
```

Navigates and writes PNG bytes directly to `PATH`. Existing files are refused
unless `--overwrite` is present. `--full-page` captures the complete document
instead of the viewport.

```sh
brimp screenshot https://example.com \
  --output example.png \
  --full-page
```

Successful screenshot commands do not write image data to standard output.

## `brimp cdp`

```text
brimp cdp [--bind HOST:PORT] [--allow-non-loopback]
```

Starts the CDP server. The default bind is `127.0.0.1:9222`. Brimp rejects a
non-loopback bind unless `--allow-non-loopback` is explicit. Every reachable
CDP client can control the browser, so exposing this server is security
sensitive. On success, standard output contains the browser WebSocket URL and
the process continues serving until it is terminated.

```sh
brimp cdp --bind 127.0.0.1:9222 --enable-canvas
```

See the [CDP API](/api/cdp/) for Playwright and Puppeteer connection examples
and the exhaustive method matrix.

## Shared navigation options

`eval` and `screenshot` accept:

| Option | Meaning |
| --- | --- |
| `--persona PATH` | Load a versioned persona JSON file before creating the page. |
| `--timeout-ms N` | Set the positive navigation timeout in milliseconds; default 30,000. |

The CDP server does not accept `--persona`; it currently uses the default
persona for all targets.

All page-creating commands accept these opt-in page options:

```text
--enable-worker
--enable-streaming-networking
--storage-path PATH [--storage-quota-bytes N]
--enable-canvas
--enable-webgl
--enable-webgpu
--enable-webaudio
--enable-webaudio-output
```

Every subsystem is disabled when its option is omitted. For `brimp cdp`, these
options become the page options for every target created by that server process.
`--enable-webaudio` uses a device-free real-time sink;
`--enable-webaudio-output` also enables WebAudio and authorizes the system audio
output device.

| Option | Enabled browser surface |
| --- | --- |
| `--enable-worker` | Dedicated workers, shared workers, service workers, and worklets. |
| `--enable-streaming-networking` | WebSocket, EventSource, streaming Fetch, and stream classes. |
| `--storage-path PATH` | Persistent origin-partitioned storage rooted at `PATH`. |
| `--storage-quota-bytes N` | Positive storage quota; requires `--storage-path` and defaults to 1 GiB. |
| `--enable-canvas` | Canvas 2D and its Skia raster backing. |
| `--enable-webgl` | WebGL 1/2 through ANGLE. |
| `--enable-webgpu` | WebGPU through `wgpu`. |
| `--enable-webaudio` | Offline and device-free realtime WebAudio. |
| `--enable-webaudio-output` | WebAudio plus authorization to open the system output device. |

## Exit codes

| Code | Category |
| ---: | --- |
| 0 | Success |
| 2 | Invalid input |
| 10 | Transport failure |
| 11 | HTTP status failure |
| 12 | Navigation failure |
| 13 | JavaScript failure |
| 14 | Timeout |
| 15 | Cancellation |
| 16 | Unsupported result |
| 17 | Closed object |
| 18 | Screenshot or internal runtime failure |

The CLI does not implement `fetch`, `extract`, `render`, or `batch` commands.
