---
title: CLI API
description: Commands, flags, output contracts, and exit codes for brimp.
---

The CLI writes structured results to standard output and diagnostics to
standard error.

## `brimp doctor`

Validates JavaScriptCore, libcurl-impersonate, and the selected transport
profile. Successful output is a JSON object.

## `brimp eval`

```text
brimp eval URL --js EXPRESSION [--persona PATH] [--timeout-ms N]
```

Navigates to `URL`, evaluates JavaScript, and prints one JSON value. The default
timeout is 30,000 milliseconds. Ctrl-C cancels navigation.

## `brimp screenshot`

```text
brimp screenshot URL --output PATH [--persona PATH] [--full-page] [--overwrite] [--timeout-ms N]
```

Navigates and writes PNG bytes directly to `PATH`. Existing files are refused
unless `--overwrite` is present. `--full-page` captures the complete document
instead of the viewport.

## `brimp cdp`

```text
brimp cdp [--bind HOST:PORT] [--allow-non-loopback]
```

Starts the CDP server. The default bind is `127.0.0.1:9222`. Brimp rejects a
non-loopback bind unless `--allow-non-loopback` is explicit. Every reachable
CDP client can control the browser, so exposing this server is security
sensitive.

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
