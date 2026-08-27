---
title: Node.js examples
description: Navigate, evaluate, capture screenshots, cancel work, and load personas.
---

## Page lifecycle

```js
const { launch } = require('@brimp/brimp')

async function main() {
  const browser = await launch()
  try {
    const page = await browser.newPage()
    try {
      await page.goto('https://example.com', { timeoutMs: 30_000 })
      console.log(await page.title())
      console.log(await page.textContent())
    } finally {
      await page.close()
    }
  } finally {
    await browser.close()
  }
}

main().catch(error => {
  console.error(error)
  process.exitCode = 1
})
```

Closing pages and browsers is idempotent.

## Structured evaluation and screenshots

```js
const fs = require('node:fs/promises')
const { launch } = require('@brimp/brimp')

const browser = await launch()
const page = await browser.newPage()

await page.goto('https://example.com')
const result = await page.evaluate(`({
  title: document.title,
  links: document.querySelectorAll('a').length
})`)
const png = await page.screenshot({ fullPage: true })

console.log(result)
await fs.writeFile('example.png', png)
await page.close()
await browser.close()
```

`evaluate()` returns JSON-compatible values. Screenshots are Node `Buffer`
objects.

## Cancel navigation

```js
const { launch } = require('@brimp/brimp')

const browser = await launch()
const page = await browser.newPage()
const controller = new AbortController()

const timer = setTimeout(() => controller.abort(), 2_000)
try {
  await page.goto('https://example.com/slow', {
    timeoutMs: 30_000,
    signal: controller.signal,
  })
} finally {
  clearTimeout(timer)
  await page.close()
  await browser.close()
}
```

Cancellation errors have `error.code === 'cancelled'`.

## Load a persona

```js
const fs = require('node:fs')
const { launch } = require('@brimp/brimp')

const browser = await launch({
  personaJson: fs.readFileSync('persona/example.json', 'utf8'),
})
```

See the [Node API](/api/node/) for the complete public surface.
