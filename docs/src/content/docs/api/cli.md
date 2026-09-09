---
title: CLI API
description: Commands, flags, output contracts, and exit codes for brimp.
---

CLI browser commands currently require macOS and a child worker. Set
`BRIMP_WORKER_PATH` to the lite worker executable or a WebKit worker bundle,
or pass `--worker-path PATH`. Build both with
`cargo build -p brimp-controller -p brimp-lite-worker`.


The CLI writes primary results to standard output or the requested path and
diagnostics to standard error.

```text
brimp doctor
brimp fetch URL [OPTIONS]
brimp crawl URL [OPTIONS]
brimp help [COMMAND]
```

Use `brimp help fetch` or `brimp help crawl` for the installed command synopsis.
For task-based walkthroughs, see [Fetch and extract pages](/examples/fetch/)
and [Crawl a website](/examples/crawl/).

## `brimp doctor`

Validates JavaScriptCore, libcurl-impersonate, and the selected transport
profile, then writes one JSON object:

```json
{"javascriptCore":"ok","libcurlImpersonate":"ok","worker":"lite-worker"}
```

## Managed WebSocket workers

All browser commands launch and own their worker. Use `--cdp`
with a Chrome executable to have the controller allocate a private profile and
loopback port, launch Chrome headless, and connect to its CDP endpoint automatically:

```sh
brimp fetch https://example.com \
  --worker-path '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
  --cdp --eval document.title
```

`--worker-args PATH` reads a JSON array of launch argument strings. File arguments
come first; repeatable `--worker-arg ARG` values are appended. Completion, errors, timeout, and cancellation
clean up the child process group. The default `framed-cdp` protocol launches
lite or WebKit workers over an inherited socketpair. Worker launch currently
requires macOS; attaching to existing browser endpoints is not supported.

## `brimp fetch`

With no output flags, `fetch` serializes the live DOM after page scripts run:

```sh
brimp fetch https://example.com
```

Select a result explicitly or infer it from the output extension:

```sh
brimp fetch URL --format raw --output response.html
brimp fetch URL --output article.html
brimp fetch URL --output article.md
brimp fetch URL --format json --output article.json
brimp fetch URL --output page.png --full-page
brimp fetch URL --eval '({title: document.title})'
brimp fetch URL --eval-file inspect.js
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
  --concurrency 2 \
  --max-pages 1000 \
  --include '/docs/**' \
  --exclude '/docs/archive/**'
```

The crawler stays on the final start origin unless `--allow-origin URL` is
repeated to expand scope. It obeys `robots.txt` unless `--ignore-robots` is
explicit, and `--delay DURATION` enforces a shared per-origin start delay.
`--format markdown|html|json`, extraction controls, waits, preparation scripts,
network/identity settings, and persistent storage options match `fetch`.

Failures are recorded without stopping unrelated pages. `--fail-fast` stops
after the current frontier, while `--allow-errors` permits a successful process
exit when terminal page records contain failures. The output directory must be
absent or empty unless `--overwrite` is present; unrelated files are never
removed.

## Browser subsystems and storage

Canvas, workers, and streaming networking are always enabled. Page-creating
commands can also enable persistent storage:

```text
--storage-path PATH [--storage-quota-bytes N]
```

Persistent storage is absent when no storage path is supplied.

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
