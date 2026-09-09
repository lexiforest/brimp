---
title: CLI and CDP examples
description: Connect Playwright or Puppeteer to a Brimp controller and find command-line guides.
---

CLI browser commands currently require macOS and a child worker. Set
`BRIMP_WORKER_PATH` to the lite worker executable or a WebKit worker bundle,
or pass `--worker-path PATH`. Build both with
`cargo build -p brimp-controller -p brimp-lite-worker`.

## Check native dependencies

```sh
brimp doctor
```

Successful output is JSON suitable for scripts:

```json
{"javascriptCore":"ok","libcurlImpersonate":"ok","worker":"lite-worker"}
```

## Retrieve and crawl pages

Use the [fetch guide](/examples/fetch/) to save HTML, extract Markdown and
metadata, evaluate JavaScript, configure personas, and capture screenshots.
Use the [crawl guide](/examples/crawl/) to follow links within a defined scope
and save pages with a manifest.

## Start the controller

```sh
brimp serve \
  --worker-path /path/to/lite-worker \
  --headless --window-size=1280,720 --port=9222
```

## Connect Playwright

Playwright attaches through its public `connectOverCDP()` API:

```js
import { chromium } from 'playwright-core'

const browser = await chromium.connectOverCDP('http://127.0.0.1:9222')

try {
  const context = browser.contexts()[0] ?? await browser.newContext()
  const page = await context.newPage()
  await page.goto('https://example.com', { waitUntil: 'load' })
  console.log(await page.evaluate(() => document.title))
  await page.close()
} finally {
  await browser.close()
}
```

Use `playwright-core`, not a Playwright browser download: Brimp is the browser
process being controlled.

## Connect Puppeteer

Puppeteer connects to the controller's HTTP discovery endpoint:

```js
const fs = require('node:fs/promises')
const puppeteer = require('puppeteer-core')

const browser = await puppeteer.connect({
  browserURL: 'http://127.0.0.1:9222',
})

const [page] = await browser.pages()
await page.goto('https://example.com')

const result = await page.evaluate(() => ({
  title: document.title,
  answer: 6 * 7,
}))
const png = await page.screenshot()

console.log(result)
await fs.writeFile('example.png', png)
await browser.disconnect()
```

Do not expose the controller to a network unless you intend to give every
reachable client control of the browser.

See the [CLI API](/api/cli/) and [CDP API](/api/cdp/) for exhaustive support.
