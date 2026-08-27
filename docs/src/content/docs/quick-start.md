---
title: Quick start
description: Render your first page with Python, Node.js, the CLI, or Puppeteer.
---

All Brimp interfaces navigate through the same runtime. Pick the one that fits
your application.

## Python

```python
import brimp

response = brimp.get("https://example.com")
response.raise_for_status()

print(response.status_code)
print(response.html)  # DOM serialized after page scripts run
```

Use a session when you need cookies, repeated navigation, evaluation, or a
screenshot:

```python
import brimp

with brimp.Session() as session:
    response = session.get("https://example.com", timeout=30)
    print(session.evaluate("document.title"))
    session.screenshot("example.png", full_page=True)
```

## Node.js

```js
const { launch } = require('@brimp/brimp')

async function main() {
  const browser = await launch()
  const page = await browser.newPage()

  await page.goto('https://example.com')
  console.log(await page.title())

  await page.close()
  await browser.close()
}

main().catch(error => {
  console.error(error)
  process.exitCode = 1
})
```

## CLI

Evaluate JavaScript after navigation:

```sh
brimp eval https://example.com --js 'document.title'
```

Capture a full-page screenshot:

```sh
brimp screenshot https://example.com \
  --output example.png \
  --full-page
```

## Puppeteer over CDP

Start Brimp's loopback CDP server:

```sh
brimp cdp --bind 127.0.0.1:9222
```

Then connect with `puppeteer-core`:

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
