---
title: Node.js examples
description: Live Page navigation, extraction, state, screenshots, and cancellation.
---

## One request

```js
const brimp = require('@brimp/brimp')

async function main() {
  const page = await brimp.get('https://example.com', {
    params: { q: ['browser', 'runtime'] },
  })
  try {
    page.raiseForStatus()
    console.log(page.text)
    console.log(page.html)
  } finally {
    await page.close()
  }
}
```

## Persistent session

```js
const { createSession } = require('@brimp/brimp')

async function main() {
  const session = await createSession()
  try {
    session.headers['X-Client'] = 'brimp'
    const page = await session.get('https://example.com')
    console.log(page.statusCode)
    console.log(await page.evaluate('document.title'))
    await page.screenshot({ path: 'example.png', fullPage: true })
  } finally {
    await session.close()
  }
}
```

## Cancel navigation

```js
const { createSession } = require('@brimp/brimp')

async function main() {
  const session = await createSession()
  const page = await session.newPage()
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), 2_000)
  try {
    await page.get('https://example.com/slow', {
      timeoutMs: 30_000,
      signal: controller.signal,
    })
  } finally {
    clearTimeout(timer)
    await session.close()
  }
}
```

Cancellation errors have `error.code === 'cancelled'`.

## Load a persona

```js
const fs = require('node:fs')
const { createSession } = require('@brimp/brimp')

async function main() {
  return createSession({
    personaJson: fs.readFileSync('persona/example.json', 'utf8'),
  })
}
```

See the [Node API](/api/node/) for the complete public surface.
