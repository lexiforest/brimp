---
title: CLI and CDP examples
description: Shell automation and a supported Puppeteer workflow.
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

`brimp eval` prints the evaluated JSON value to standard output and diagnostics
to standard error:

```sh
brimp eval https://example.com \
  --js '({title: document.title, links: document.querySelectorAll("a").length})'
```

Set a navigation timeout in milliseconds when the 30-second default is not
appropriate:

```sh
brimp eval https://example.com \
  --timeout-ms 10000 \
  --js 'document.body.textContent.trim()'
```

## Use a persona

```sh
brimp eval https://example.com \
  --persona persona/example.json \
  --js '({ua: navigator.userAgent, platform: navigator.platform})'
```

## Protect screenshot output

```sh
brimp screenshot https://example.com \
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

Do not expose the server to a network unless you intend to give every reachable
client control of the browser. Non-loopback binds require
`--allow-non-loopback` and print a security warning.

See the [CLI API](/api/cli/) and [CDP API](/api/cdp/) for exhaustive support.
