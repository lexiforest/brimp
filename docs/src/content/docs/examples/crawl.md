---
title: Crawl a website
description: Set crawl limits, control discovery, and save pages with a JSON Lines manifest.
---

CLI browser commands require macOS and a child worker. Follow [Install](/install/),
then set `BRIMP_WORKER_PATH` to the worker executable or pass
`--worker-path PATH` with each command.

`brimp crawl` follows links and saves pages using the same navigation and
extraction behavior as [`brimp fetch`](/examples/fetch/). It saves Markdown
by default.

```sh
brimp crawl https://example.com
```

| Setting | Default |
| --- | --- |
| Output directory | `./brimp-crawl` |
| Maximum link depth | `2` |
| Concurrent pages | `2` |
| Maximum pages | `1000` |
| Scope | Same origin as the final start URL |
| Robots policy | Obey `robots.txt` |
| Output | Markdown and a JSON Lines manifest |

The output directory must be absent or empty. `--overwrite` permits replacing
page files and the manifest but never removes unrelated files.

## Set crawl limits

```sh
brimp crawl https://example.com --depth 5
brimp crawl https://example.com --concurrency 5
brimp crawl https://example.com --max-pages 250
brimp crawl https://example.com --output-dir ./reference
```

Each crawl slot processes one URL at a time in its own page. Pages share one
browser context and cookie jar. Increase
`--concurrency` to navigate and extract more pages concurrently.

## Control which links are followed

The crawler extracts links from the rendered live DOM after the configured wait
condition. Fragments are removed, URLs are canonicalized, and duplicate URLs
are visited once.

```sh
brimp crawl https://example.com/docs \
  --include '/docs/**' \
  --exclude '/docs/archive/**'

brimp crawl https://example.com \
  --allow-origin https://static.example.com
```

Cross-origin links are ignored unless their origin is explicitly allowed.
Redirects outside the allowed origins are recorded as skipped.

## Configure robots rules and pacing

```sh
brimp crawl https://example.com --ignore-robots
brimp crawl https://example.com --delay 500ms
```

Robots rules are fetched through Brimp's configured transport and cached per
origin. `--delay` is a minimum start-to-start delay per origin, shared across
workers.

## Save pages and inspect results

```sh
brimp crawl https://example.com --format markdown
brimp crawl https://example.com --format html
brimp crawl https://example.com --format json
```

Output paths are deterministic and remain inside `--output-dir`. The crawler
writes pages atomically and maintains `manifest.jsonl` with one terminal record
per discovered URL:

```json
{"url":"https://example.com/docs/","finalUrl":"https://example.com/docs/","depth":1,"status":200,"output":"docs/index.md","ok":true,"error":null,"skipped":null}
```

## Handle failures and cancellation

Errors are recorded in the manifest and do not stop unrelated pages by default.
A crawl with failed pages exits with a nonzero status. Use `--fail-fast` to stop
after the current frontier, or `--allow-errors` to allow a successful exit despite
page failures:

```sh
brimp crawl https://example.com --fail-fast
brimp crawl https://example.com --allow-errors
```

Ctrl-C stops scheduling new URLs, cancels in-flight work, finishes the manifest,
and exits with code `15`.

Crawls also accept fetch options for waits, preparation scripts, extraction,
network requests, personas, persistent storage, and per-page timeouts. See the
[fetch guide](/examples/fetch/) for examples and the [CLI reference](/api/cli/)
for exit codes.
