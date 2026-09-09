---
title: Fetch and extract pages
description: Retrieve live HTML, extract Markdown, evaluate JavaScript, and capture screenshots with brimp fetch.
---

CLI browser commands require macOS and a child worker. Follow [Install](/install/),
then set `BRIMP_WORKER_PATH` to the worker executable or pass
`--worker-path PATH` with each command.

Results go to standard output or the path selected by `--output`. Diagnostics
and progress go to standard error. Existing files require `--overwrite` before
they can be replaced. Metadata and relative links use the final URL after redirects.

## Save a page

With no output option, `fetch` writes the live HTML after page scripts run to
standard output:

```sh
brimp fetch https://example.com
```

Write the rendered DOM to a file:

```sh
brimp fetch https://example.com --output example.html
```

Write the original response bytes rather than the rendered DOM:

```sh
brimp fetch https://example.com --format raw --output response.html
```

Output formats are explicit with `--format`, or inferred from a file extension
when unambiguous:

| Format | Inferred extensions | Output |
| --- | --- | --- |
| `html` | `.html`, `.htm` | Live DOM serialized after scripts and waits. |
| `markdown` | `.md`, `.markdown` | Defuddle extraction from the live DOM. |
| `json` | `.json` | Defuddle content and metadata as one JSON object. |
| `png` | `.png` | CPU-rendered page screenshot. |
| `raw` | Never inferred | Original response bytes. |

When `--output -` is used, `--format` is required because there is no extension
to inspect:

```sh
brimp fetch https://example.com --format markdown --output -
```

## Extract Markdown and metadata

The `.md` output path selects Markdown extraction:

```sh
brimp fetch https://example.com/article --output article.md
```

Write the complete Defuddle result, including metadata and Markdown content:

```sh
brimp fetch https://example.com/article --format json --output article.json
```

Select known content instead of running automatic main-content detection:

```sh
brimp fetch https://example.com/article \
  --format markdown \
  --content 'article.post' \
  --output article.md
```

Remove images, specify the extraction language, or include debugging details:

```sh
brimp fetch https://example.com --format markdown --remove-images
brimp fetch https://example.com --format markdown --language en-US
brimp fetch https://example.com --format json --extract-debug
```

Extraction uses the bundled Defuddle browser script against the live `document`,
including changes made by page scripts. It runs locally; third-party
asynchronous extractors are disabled.

## Evaluate JavaScript and prepare the page

Evaluate an expression after navigation and print its JSON-compatible result:

```sh
brimp fetch https://example.com --eval 'document.title'
brimp fetch https://example.com --eval '({ title: document.title, links: document.links.length })'
```

Read an expression from a file:

```sh
brimp fetch https://example.com --eval-file inspect.js
```

Run a preparation script before producing HTML, Markdown, JSON, or a screenshot:

```sh
brimp fetch https://example.com/article \
  --script expand-article.js \
  --output article.md
```

`--eval` and `--eval-file` select the result and cannot be combined with
`--format` or `--output`. `--script` may be repeated; scripts
run in argument order before output is captured.

## Capture a screenshot

The output extension selects PNG:

```sh
brimp fetch https://example.com --output example.png
brimp fetch https://example.com --output example.png --full-page
```

An explicit format is useful when writing binary output to stdout:

```sh
brimp fetch https://example.com --format png --output - > example.png
```

## Wait for page content

The default lifecycle wait is `load`:

```sh
brimp fetch https://example.com --wait domcontentloaded
brimp fetch https://example.com --wait load
brimp fetch https://example.com --wait networkidle
```

A numeric wait means a fixed number of seconds after the lifecycle condition:

```sh
brimp fetch https://example.com --wait 30
```

Wait for application state after lifecycle completion:

```sh
brimp fetch https://example.com --wait-selector 'article[data-ready]'
brimp fetch https://example.com --wait-selector '#results' --timeout 45s
```

`networkidle` means no active HTTP requests for 500 ms. The quiet window can be
changed explicitly:

```sh
brimp fetch https://example.com --wait networkidle --network-idle 1s
```

Fixed waits and selector waits are cancellation-aware. The operation timeout
includes navigation, waits, scripts, extraction, rendering, and output.

## Configure network requests and identity

Proxy traffic through the same libcurl-impersonate transport used by the page:

```sh
brimp fetch https://example.com --proxy http://127.0.0.1:1080
brimp fetch https://example.com --proxy socks5://127.0.0.1:1080
```

Add request state:

```sh
brimp fetch https://example.com --header 'X-Agent: research'
brimp fetch https://example.com --cookie 'session=abc123'
brimp fetch https://example.com --persona crates/worker/src/persona/example.json
brimp fetch https://example.com --ca-bundle internal-ca.pem
```

Repeat `--header` and `--cookie` to supply multiple values. Persona-owned identity headers cannot be
overridden independently; configure them in the persona file.

## Persist browser storage

The lite worker always enables canvas, workers, and streaming networking.
To persist browser storage, supply a profile path:

```sh
brimp fetch https://example.com --storage-path ./profile --storage-quota-bytes 1073741824
```

For batch retrieval, see [Crawl a website](/examples/crawl/). The
[CLI reference](/api/cli/) lists command options and exit codes.
