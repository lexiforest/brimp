---
title: CLI API
description: Commands, flags, output contracts, and exit codes for brimp.
---

The CLI writes primary results to standard output or the requested path and
diagnostics to standard error.

```text
brimp doctor
brimp get URL [OPTIONS]
brimp crawl URL [OPTIONS]
brimp cdp [OPTIONS]
brimp help [COMMAND]
```

Use `brimp help get`, `brimp help crawl`, or `brimp help cdp` for the installed
command synopsis.

## `brimp doctor`

Validates JavaScriptCore, libcurl-impersonate, and the selected transport
profile, then writes one JSON object:

```json
{"javascriptCore":"ok","libcurlImpersonate":"ok","profile":"chrome150"}
```

## `brimp get`

With no output flags, `get` serializes the live DOM after page scripts run:

```sh
brimp get https://example.com
```

Select a result explicitly or infer it from the output extension:

```sh
brimp get URL --format raw --output response.html
brimp get URL --output article.html
brimp get URL --output article.md
brimp get URL --format json --output article.json
brimp get URL --output page.png --full-page
brimp get URL --eval '({title: document.title})'
brimp get URL --eval-file inspect.js
```

`raw` is the original response body. `html` is the rendered DOM. `markdown` and
`json` run the pinned Defuddle browser bundle against the live DOM. `png`
captures the page. `--output -` requires an explicit format. Evaluation is a
result mode and cannot be combined with `--format` or `--output`.

Preparation, extraction, and wait controls include:

```text
--script PATH                         repeatable, runs in order
--content SELECTOR
--remove-images
--language BCP47
--extract-debug
--wait domcontentloaded|load|networkidle|SECONDS
--wait-selector SELECTOR
--network-idle DURATION
--timeout DURATION                    accepts ms, s, or m
```

Network and identity controls include repeatable `--header 'NAME: VALUE'` and
`--cookie 'NAME=VALUE'`, plus `--proxy URL`, `--persona PATH`, and
`--ca-bundle PATH`. Cookie options are inserted once into the browser-context
cookie jar and then follow normal domain, path, redirect, and expiry rules;
they are not replayed as static headers. Persona-owned identity headers cannot be overridden
independently. Existing output files are refused unless `--overwrite` is
explicit. Ctrl-C and the operation timeout cover navigation, waits, scripts,
extraction, rendering, and output.

## `brimp crawl`

`crawl` is a bounded deterministic breadth-first pipeline. Markdown is the
default output format and `manifest.jsonl` contains one terminal record per
discovered URL.

```sh
brimp crawl https://example.com/docs \
  --output-dir ./reference \
  --depth 2 \
  --workers 2 \
  --max-pages 1000 \
  --include '/docs/**' \
  --exclude '/docs/archive/**'
```

The crawler stays on the final start origin unless `--allow-origin URL` is
repeated to expand scope. It obeys `robots.txt` unless `--ignore-robots` is
explicit, and `--delay DURATION` enforces a shared per-origin start delay.
`--format markdown|html|json`, extraction controls, waits, preparation scripts,
network/identity settings, and opt-in page subsystems match `get`.

Failures are recorded without stopping unrelated pages. `--fail-fast` stops
after the current frontier, while `--allow-errors` permits a successful process
exit when terminal page records contain failures. The output directory must be
absent or empty unless `--overwrite` is present; unrelated files are never
removed.

## `brimp cdp`

```text
brimp cdp [--bind HOST:PORT] [--allow-non-loopback] [PAGE OPTIONS]
```

The default bind is `127.0.0.1:9222`. A non-loopback bind is rejected unless
explicitly allowed because every reachable client can control the browser. On
success, stdout contains the browser WebSocket URL and the process serves until
terminated.

## Optional page subsystems

Page-creating commands accept these opt-in controls:

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

Every subsystem is absent when omitted. `--enable-webaudio` uses a device-free
sink; `--enable-webaudio-output` additionally authorizes the system output
device.

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
| 18 | Extraction, screenshot, or internal runtime failure |
