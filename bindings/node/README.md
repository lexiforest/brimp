# Brimp Node binding

Brimp exposes an asynchronous request/response API backed by the shared native
browser runtime. It runs in process and does not start a CDP server.

```js
const brimp = require('@brimp/brimp')

async function main() {
  const response = await brimp.get('https://example.com')
  console.log(response.statusCode)
  console.log(response.text) // original HTTP response
  console.log(response.html) // DOM after JavaScript
}
```

Sessions own shared cookies while pages own documents, connections, an optional
immutable proxy, and `AbortSignal`-cancellable navigation:

```js
async function main() {
  const session = await brimp.createSession()
  try {
    const page = await session.newPage({ proxy: 'socks5h://127.0.0.1:1080' })
    const response = await page.get('https://example.com', { timeoutMs: 30_000 })
    response.raiseForStatus()
    console.log(await page.evaluate('document.title'))
    const article = await page.extract({ contentSelector: 'main' })
    console.log(article.contentMarkdown)
    await page.hover('#menu')
    await page.type('#name', 'agent')
    await page.click('#submit')
    await page.screenshot({ path: 'page.png', fullPage: true })
  } finally {
    await session.close()
  }
}
```

`page.extract()` runs the vendored Defuddle browser bundle against the live,
post-JavaScript document. It does not create a jsdom document or make another
network request.

Worker, streaming-networking, persistent-storage, Canvas 2D, WebGL, WebGPU,
and WebAudio APIs are disabled by default. Enable only the required surfaces
through `createSession(options)`. `enableWebAudio` uses a device-free sink;
`enableWebAudioOutput` also enables WebAudio and authorizes the system output
device.

This API is for asynchronous rendered-page extraction. Use `brimp cdp` when a
Puppeteer or Playwright browser/page interface is required. See `SUPPORT.md`
for the exact tested surface.
