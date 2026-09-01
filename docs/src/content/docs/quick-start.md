---
title: Quick start
description: Render your first page with Python, Node.js, the CLI, or Puppeteer.
---

All Brimp interfaces navigate through the same runtime. Pick the one that fits
your application.

## Python

```python
import brimp

with brimp.get("https://example.com") as page:
    page.raise_for_status()
    print(page.status_code)
    print(page.html)  # current live DOM
```

Use a session when you need cookies, repeated navigation, evaluation, or a
screenshot:

```python
import brimp

with brimp.Session(pool_size=8) as session:
    with session.get("https://example.com", timeout=30) as page:
        print(page.evaluate("document.title"))
        print(page.extract()["contentMarkdown"])
        page.hover("#menu")
        page.type("#name", "agent")
        page.click("#submit")
        page.screenshot("example.png", full_page=True)
```

## Node.js

```js
const { createSession } = require('@brimp/brimp')

async function main() {
  const session = await createSession()
  try {
    const page = await session.get('https://example.com')
    console.log(page.statusCode, page.html)
    console.log(await page.evaluate('document.title'))
    console.log((await page.extract()).contentMarkdown)
    await page.hover('#menu')
    await page.type('#name', 'agent')
    await page.click('#submit')
  } finally {
    await session.close()
  }
}

main().catch(error => {
  console.error(error)
  process.exitCode = 1
})
```

## CLI

Extract Markdown from the live page or evaluate JavaScript after navigation:

```sh
brimp get https://example.com --output example.md
brimp get https://example.com --eval 'document.title'
```

Capture a full-page screenshot:

```sh
brimp get https://example.com \
  --output example.png \
  --full-page
```

## Playwright or Puppeteer over CDP

Start Brimp's loopback CDP server:

```sh
brimp cdp --bind 127.0.0.1:9222
```

Connect with `playwright-core`:

```js
import { chromium } from 'playwright-core'

const browser = await chromium.connectOverCDP('http://127.0.0.1:9222')
const context = browser.contexts()[0] ?? await browser.newContext()
const page = await context.newPage()
await page.goto('https://example.com')
console.log(await page.evaluate(() => document.title))
await browser.close()
```

Or connect with `puppeteer-core`:

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

The CDP server supports a checked protocol subset. Consult the [CDP API
reference](/api/cdp/) before adapting a larger Puppeteer workflow.
