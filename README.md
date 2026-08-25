# Brimp

Brimp is a lightweight, headless browser for agents. It combines JavaScriptCore with
Blitz's DOM implementation and curl-impersonate as the network stack.

If you are familiar with `curl_cffi`, you can simply treat brimp as **curl_cffi with
JavaScript enabled**. Brimp offers both python and nodejs bindings.

Brimp is inspired by Cloudflare's [kitesurf](https://blog.cloudflare.com/kitesurf/).

## Why

Getting the html with `curl_cffi` is easy, but oftentimes, you need to render the page
with JavaScript, and anti-bot services are consistently probing if you have a valid
browser environment, hence JavaScript fingerprints.

Brimp, short for browser-impersonate, tries to add the missing JS and DOM layer for
curl_cffi.

## Features

- Page rendered with DOM and JS, so you don't get the empty html placeholder.
- `curl-impersonate` as the network stack, providing perfect Ja3/TLS fingerprints.
- The familiar `curl_cffi` API and experience from the same maintainer.
- Also supports CDP, drop-in replacement for heavy headless browsers.
- Much faster than playwright with Chromium, on par with popular alternatives.
- Python and Nodejs bindings.
- Pre-compiled, so you don't have to compile on your machine.

For performance comparisons, see the [benchmark](#benchmark).

## Install

Release packages currently target macOS arm64:

```sh
python -m pip install brimp
npm install @brimp/brimp
```

The standalone `brimp` and `brimp-cdp` executables are produced by the source
build below.

## Usage

### CLI

Check the native runtime, evaluate JavaScript after navigation, or capture a
PNG from the command line:

```sh
brimp doctor
brimp eval https://example.com --js 'document.title'
brimp screenshot https://example.com --output example.png --full-page
```

### Python

The Python and Node bindings are native extensions loaded into their host processes.
They do not start a server and are not CDP clients, making them more lightweight.

```python
import asyncio
import brimp

async def main():
    browser = await brimp.launch()
    page = await browser.new_page()
    await page.goto("https://example.com")
    print(await page.evaluate("document.title"))
    await page.close()
    await browser.close()

asyncio.run(main())
```

### Node.js

```js
const brimp = require('@brimp/brimp')

async function main() {
  const browser = await brimp.launch()
  const page = await browser.newPage()
  await page.goto('https://example.com')
  console.log(await page.evaluate('document.title'))
  await page.close()
  await browser.close()
}

main().catch(error => {
  console.error(error)
  process.exitCode = 1
})
```

### With CDP clients

Start the bounded loopback CDP server and connect with `puppeteer-core`:

```sh
brimp-cdp --bind 127.0.0.1:9222
```

```js
const puppeteer = require('puppeteer-core')

const browser = await puppeteer.connect({
  browserURL: 'http://127.0.0.1:9222',
})
const [page] = await browser.pages()
await page.goto('https://example.com')
console.log(await page.evaluate(() => document.title))
await browser.disconnect()
```

Brimp implements the CDP subset needed by this workflow, not the complete
Chrome DevTools Protocol.

## Building from source

As in the current stage, we only support building on local macOS environment.

### Prerequisites

The current JSC binding targets macOS and dynamically links a JSCOnly build at:

```text
../WebKit/WebKitBuild/Release/JavaScriptCore.framework
```

Set `BRIMP_JSC_FRAMEWORK_DIR` to use a different framework directory. The
Brimp-owned network transport links `libcurl-impersonate` from `/usr/local/lib`;
set `BRIMP_CURL_LIB_DIR` to use another location.

## Benchmark

Here is the latest complete fixture benchmark for Brimp and the other agentic
browsers. Each result validates the live DOM; incorrect samples are excluded
from latency and memory aggregates. Cold results launch a fresh process, while
warm results reuse a browser and create a fresh page.

![Brimp browser benchmark comparison](benchmark/2026-08-25.svg)

To run the benchmark:

```sh
cargo run -p brimp-benchmark --release -- --samples 20
uv run --project benchmark/performance --no-sync python benchmark/performance/bench.py
uv run --project benchmark/performance --no-sync python benchmark/memory/memory.py
cargo run -p brimp-cdp -- --bind 127.0.0.1:9222
```

## Development

### Architecture

The implemented runtime supports:

- static HTML, JavaScript DOM mutation, style/layout queries, and CPU PNG screenshots;
- HTTP(S) navigation through a transport-neutral `ResourceLoader`;
- linked CSS, common raster images, classic inline/external scripts, `defer`, and `async`;
- parser pausing around blocking scripts;
- events, timers, microtasks, Promise-based `fetch`, cookies, `Location`, and `Navigator`.

The canonical owner-thread automation API is exposed through:

- the `brimp` CLI for evaluation and screenshots;
- asynchronous in-process Python and Node native bindings; and
- a bounded loopback CDP server for the checked Puppeteer workflow.

All four interfaces delegate navigation, JavaScript, lifecycle, and screenshots
to `web-runtime`; none contains a second browser implementation.

### Testing

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
python3 -m unittest discover -s benchmark/performance -p 'test_*.py' -v
python3 -m unittest discover -s benchmark/memory -p 'test_*.py' -v
./bindings/package-test.sh
./crates/cdp/puppeteer-test.sh
```

Current release artifacts target macOS arm64. Python and Node packages bundle
the arm64 libcurl-impersonate dylib and use macOS system frameworks at runtime;
source builds use the configurable native discovery paths above. See each
interface's `SUPPORT.md` for its exact tested surface.
