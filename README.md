# Brimp

Brimp is a unified browser interface for agents. It has two builtin runtime

Brimp is a lightweight, headless browser for agents. It combines JavaScriptCore with
Blitz's DOM implementation and curl-impersonate as the network stack.

Documentation: [docs.brimp.ai](https://docs.brimp.ai)

Key references:

- [CLI commands](https://docs.brimp.ai/api/cli/)
- [CDP support matrix and Playwright connection](https://docs.brimp.ai/api/cdp/)
- [JavaScriptCore integration and direct page evaluation](https://docs.brimp.ai/architecture/javascript-runtime/)
- [Subsystem implementation](https://docs.brimp.ai/architecture/subsystems/)

## Why

Getting the html with `curl_cffi` is easy, but oftentimes, you need to render the page
with JavaScript, and anti-bot services are consistently probing if you have a valid
browser environment, hence JavaScript fingerprints.

Brimp, short for browser-impersonate, tries to add the missing JS and DOM layer for
curl_cffi.

## Features

- Page rendered with DOM and JS, so you don't get the empty html placeholder.
- `curl-impersonate` as the network stack, providing perfect Ja3/TLS, http 2&3 fingerprints.
- Provides a CDP worker for the Brimp controller (`brimp serve`).
- Much faster than playwright with Chromium, on par with popular alternatives.
- Pre-compiled, so you don't have to compile on your machine.
- MIT licensed.

||chromium|camoufox|cloakbrowser|lightpanda|obscura|brimp|
|---|---|---|---|---|---|---|
|JS & DOM|✅|✅|✅|✅|✅|✅|
|http/3|✅|✅|✅|❌|❌|✅|
|CDP|✅|✅|✅|☑️<sup>1</sup>|☑️<sup>1</sup>|☑️<sup>1</sup>|
|screenshot|✅|✅|✅|❌|✅|✅|
|JS engine|V8|SpiderMonkey|V8|V8|V8|JSC|
|open source|✅|✅|❌|☑️<sup>2</sup>️|✅|✅|
|ja3 fingerprints|☑️<sup>3</sup>️|☑️<sup>3</sup>️|☑️<sup>3</sup>️|❌|✅|✅|
|fast?|🐢|🐢|🐢|🐇|🐇|🐇|

<small>
Notes:
<ol>
<li>Only a common subset was implemented.</li>
<li>Lightpanda is under the AGPL license.</li>
<li>Fingerprints are fixed to the browser version</li>
</ol>
</small>

## Install

For humans:

```
curl https://brimp.ai/install.sh | bash
```

For agents:

Paste this prompt:

```plain
Install this tool: https://github.com/lexiforest/brimp
```

You can also download the prebuilt binary tarballs from github release page.

## Usage

### CLI

The CLI launches and manages its browser workers. Select a lite or
WebKit worker with `--worker-path` or `BRIMP_WORKER_PATH`; use
`--cdp` to launch a WebSocket CDP worker such as Chrome. The
controller discovers and connects to its child automatically. For a lite worker:

```sh
export BRIMP_WORKER_PATH=/path/to/lite-worker
```

Check the worker, extract a live page, evaluate JavaScript, or capture a PNG:

```sh
brimp doctor
brimp fetch https://example.com --output example.md
brimp fetch https://example.com --eval 'document.title'
brimp fetch https://example.com --output example.png --full-page
brimp fetch https://example.com --persona crates/worker/src/persona/example.json --eval 'navigator.userAgent'
```

### With CDP clients

Start the Brimp controller (`brimp serve`) with `lite-worker`, then connect with
`playwright-core`:

```sh
brimp serve \
  --worker-path /path/to/lite-worker \
  --headless --window-size=1280,720 --port=9222
```

```js
import { chromium } from 'playwright-core'

const browser = await chromium.connectOverCDP('http://127.0.0.1:9222')
const context = browser.contexts()[0] ?? await browser.newContext()
const page = await context.newPage()
await page.goto('https://example.com')
console.log(await page.evaluate(() => document.title))
await browser.close()
```

Brimp implements the CDP subset needed by this workflow, not the complete
Chrome DevTools Protocol. Puppeteer is also supported through
`puppeteer.connect({ browserURL: 'http://127.0.0.1:9222' })`.

## Building from source

### Prerequisites

Source builds dynamically link JavaScriptCore and curl-impersonate. Set
`BRIMP_JSC_LIB_DIR` and `BRIMP_CURL_LIB_DIR` to their platform-specific library
directories. See the [native prerequisites](docs/src/content/docs/development.md#native-prerequisites)
for SDK preparation and library layouts.

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
- the framed CDP worker used by the Brimp controller (`brimp serve`).

The lite worker delegates browser operations to `runtime`.
The CLI and controller communicate with worker processes over framed CDP and
never initialize the browser runtime themselves.

### CI

The project is tested and built automatically on GitHub Actions. All PR must pass
the automatic tests.

### Testing

```sh
cargo build -p brimp-controller -p brimp-lite-worker
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
