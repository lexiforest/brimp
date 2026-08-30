---
title: CLI and CDP examples
description: Shell automation and supported Playwright and Puppeteer workflows.
---

## Check native dependencies

```sh
brimp doctor
```

Successful output is JSON suitable for scripts:

```json
{"javascriptCore":"ok","libcurlImpersonate":"ok","profile":"chrome150"}
```

## Extract structured data

`brimp get --eval` prints the evaluated JSON value to standard output and diagnostics
to standard error:

```sh
brimp get https://example.com \
  --eval '({title: document.title, links: document.querySelectorAll("a").length})'
```

Set an operation timeout when the 30-second default is not
appropriate:

```sh
brimp get https://example.com \
  --timeout 10s \
  --eval 'document.body.textContent.trim()'
```

## Use a persona

```sh
brimp get https://example.com \
  --persona persona/example.json \
  --eval '({ua: navigator.userAgent, platform: navigator.platform})'
```

## Protect screenshot output

```sh
brimp get https://example.com \
  --output example.png \
  --full-page
```

Brimp refuses to replace an existing file unless `--overwrite` is explicit.

## Connect Puppeteer

Start the server on its loopback default:

```sh
brimp cdp
```

The first output line is the browser WebSocket endpoint. Puppeteer can discover
it through the HTTP endpoint:

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

Do not expose the server to a network unless you intend to give every reachable
client control of the browser. Non-loopback binds require
`--allow-non-loopback` and print a security warning.

See the [CLI API](/api/cli/) and [CDP API](/api/cdp/) for exhaustive support.
